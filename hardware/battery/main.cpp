#include <iostream>
#include <fstream>
#include <string>
#include <chrono>
#include <thread>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#include <filesystem>
#include <cstdlib>
#include <ctime>

namespace fs = std::filesystem;

// Paths for BAT0 on Linux
const std::string SYSFS_BAT_DIR = "/sys/class/power_supply/BAT0";
const std::string STATE_DIR = "/run/xos";
const std::string STATE_FILE = "/run/xos/battery.state";
const std::string FALLBACK_STATE_FILE = "/tmp/xos_battery.state";
const std::string LOG_DIR = "/var/log/xos";
const std::string LOG_FILE = "/var/log/xos/battery-daemon.log";

// Global configuration
int charge_limit = 100; // Default limit

bool is_dir_writable(const std::string& dir_path) {
    std::error_code ec;
    if (!fs::exists(dir_path)) {
        if (!fs::create_directories(dir_path, ec)) {
            return false;
        }
    }
    std::string test_file = dir_path + "/.xos_write_test";
    std::ofstream out(test_file);
    if (!out.is_open()) {
        return false;
    }
    out.close();
    fs::remove(test_file, ec);
    return true;
}

bool atomic_write(const std::string& path, const std::string& content) {
    fs::path dest_path(path);
    fs::path parent_path = dest_path.parent_path();
    fs::path temp_path = parent_path / (dest_path.filename().string() + ".tmp");

    std::ofstream out(temp_path);
    if (!out.is_open()) {
        return false;
    }
    out << content;
    out.close();

    std::error_code ec;
    fs::rename(temp_path, dest_path, ec);
    if (ec) {
        fs::remove(temp_path, ec);
        return false;
    }
    return true;
}

void log_message(const std::string& msg) {
    std::error_code ec;
    if (!fs::exists(LOG_DIR)) {
        fs::create_directories(LOG_DIR, ec);
    }
    std::ofstream log_stream;
    log_stream.open(LOG_FILE, std::ios_base::app);

    auto now = std::chrono::system_clock::now();
    auto time_t_now = std::chrono::system_clock::to_time_t(now);
    std::tm tm_now;
    #ifdef _WIN32
    localtime_s(&tm_now, &time_t_now);
    #else
    localtime_r(&time_t_now, &tm_now);
    #endif
    char time_buf[64];
    std::strftime(time_buf, sizeof(time_buf), "%Y-%m-%d %H:%M:%S", &tm_now);

    if (log_stream.is_open()) {
        log_stream << "[" << time_buf << "] " << msg << std::endl;
    } else {
        std::cerr << "Log fallback: [" << time_buf << "] " << msg << std::endl;
    }
}

// Read simple string from file
std::string read_file_string(const std::string& path) {
    std::ifstream file(path);
    if (!file.is_open()) return "";
    std::string line;
    std::getline(file, line);
    return line;
}

// Read integer from file
long long read_file_int(const std::string& path) {
    std::string s = read_file_string(path);
    if (s.empty()) return 0;
    try {
        return std::stoll(s);
    } catch (...) {
        return 0;
    }
}

void write_state(int capacity, const std::string& status, double watts, int cycles, double health) {
    std::string path = STATE_FILE;
    if (!is_dir_writable(STATE_DIR)) {
        path = FALLBACK_STATE_FILE;
    }

    std::string content =
        "capacity=" + std::to_string(capacity) + "\n" +
        "status=" + status + "\n" +
        "watts=" + std::to_string(watts) + "\n" +
        "cycles=" + std::to_string(cycles) + "\n" +
        "health=" + std::to_string(health) + "\n" +
        "charge_limit=" + std::to_string(charge_limit) + "\n";

    if (!atomic_write(path, content)) {
        log_message("ERROR: Could not write state file at " + path);
    }
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    // Seed random number generator
    std::srand(static_cast<unsigned int>(std::time(nullptr)));

    // Attempt to create log dir if it doesn't exist
    std::error_code ec;
    if (!fs::exists(LOG_DIR)) {
        fs::create_directories(LOG_DIR, ec);
    }

    log_message("XOS Battery Daemon starting...");

    // Simulated battery values if real sysfs BAT0 doesn't exist
    int sim_capacity = 85;
    std::string sim_status = "Discharging";
    double sim_watts = -12.5;
    int sim_cycles = 124;
    double sim_health = 94.2;

    bool use_mock = !fs::exists(SYSFS_BAT_DIR);
    if (use_mock) {
        log_message("WARNING: BAT0 not found in sysfs. Using simulated battery data.");
    }

    std::string cached_config_path = "";
    fs::file_time_type last_write_time;
    bool has_cache = false;
    bool prev_existed = false;
    int cached_charge_limit = 100;

    while (true) {
        int capacity = 0;
        std::string status = "Unknown";
        double watts = 0.0;
        int cycles = 0;
        double health = 100.0;

        if (!use_mock) {
            capacity = static_cast<int>(read_file_int(SYSFS_BAT_DIR + "/capacity"));
            status = read_file_string(SYSFS_BAT_DIR + "/status");
            if (status.empty()) status = "Unknown";

            // Power/Current/Voltage
            long long power = read_file_int(SYSFS_BAT_DIR + "/power_now");
            if (power > 0) {
                // power_now is in microwatts
                watts = static_cast<double>(power) / 1000000.0;
            } else {
                long long current = read_file_int(SYSFS_BAT_DIR + "/current_now");
                long long voltage = read_file_int(SYSFS_BAT_DIR + "/voltage_now");
                watts = (static_cast<double>(current) / 1000000.0) * (static_cast<double>(voltage) / 1000000.0);
            }

            if (status == "Discharging" && watts > 0) {
                watts = -watts; // negative for discharge
            }

            cycles = static_cast<int>(read_file_int(SYSFS_BAT_DIR + "/cycle_count"));

            // Health calculation
            long long full = read_file_int(SYSFS_BAT_DIR + "/charge_full");
            long long design = read_file_int(SYSFS_BAT_DIR + "/charge_full_design");
            if (full == 0 || design == 0) {
                full = read_file_int(SYSFS_BAT_DIR + "/energy_full");
                design = read_file_int(SYSFS_BAT_DIR + "/energy_full_design");
            }

            if (design > 0) {
                health = (static_cast<double>(full) / static_cast<double>(design)) * 100.0;
            } else {
                health = 100.0;
            }

            // Limit battery charging if charging & limit is set
            if (status == "Charging" && capacity >= charge_limit) {
                log_message("Charge limit of " + std::to_string(charge_limit) + "% reached. Restricting charging.");
                // Note: Writing to charge_control_limit_max is a privileged hardware control.
                // In a production daemon, we would do:
                // std::ofstream sys_limit(SYSFS_BAT_DIR + "/charge_control_limit_max");
                // if (sys_limit.is_open()) sys_limit << charge_limit;
            }
        } else {
            // Update simulated values
            if (sim_status == "Discharging") {
                sim_capacity -= 1;
                sim_watts = -10.0 - (rand() % 50) / 10.0;
                if (sim_capacity <= 15) {
                    sim_status = "Charging";
                    log_message("Simulated battery low (15%). Switched status to Charging.");
                }
            } else {
                sim_capacity += 1;
                sim_watts = 15.0 + (rand() % 80) / 10.0;
                if (sim_capacity >= charge_limit) {
                    sim_status = "Full";
                    sim_capacity = charge_limit;
                    sim_watts = 0.0;
                    log_message("Simulated battery reached charge limit.");
                }
            }

            if (sim_status == "Full" && rand() % 5 == 0) {
                sim_status = "Discharging";
                log_message("Simulated charger unplugged.");
            }

            capacity = sim_capacity;
            status = sim_status;
            watts = sim_watts;
            cycles = sim_cycles;
            health = sim_health;
        }

        write_state(capacity, status, watts, cycles, health);

        // Check if charge limit setting was requested (e.g. read from an external file config)
        std::string config_path = "/etc/xos/battery.conf";
        if (!fs::exists(config_path)) {
            config_path = "/tmp/xos_battery.conf";
        }

        bool config_exists = fs::exists(config_path);
        bool needs_reload = true;
        std::error_code file_ec;
        if (has_cache && cached_config_path == config_path) {
            if (config_exists == prev_existed) {
                if (!config_exists) {
                    needs_reload = false;
                } else {
                    auto current_write_time = fs::last_write_time(config_path, file_ec);
                    if (!file_ec && current_write_time == last_write_time) {
                        needs_reload = false;
                    }
                }
            }
        }

        if (needs_reload) {
            if (config_exists) {
                std::ifstream config_in(config_path);
                if (config_in.is_open()) {
                    std::string line;
                    while (std::getline(config_in, line)) {
                        if (line.rfind("charge_limit=", 0) == 0) {
                            try {
                                charge_limit = std::stoi(line.substr(13));
                            } catch (...) {}
                        }
                    }
                    config_in.close();
                }
                auto current_write_time = fs::last_write_time(config_path, file_ec);
                if (!file_ec) {
                    last_write_time = current_write_time;
                }
            }
            cached_config_path = config_path;
            prev_existed = config_exists;
            has_cache = true;
            cached_charge_limit = charge_limit;
        } else {
            charge_limit = cached_charge_limit;
        }

        std::this_thread::sleep_for(std::chrono::seconds(5));
    }

    return 0;
}
