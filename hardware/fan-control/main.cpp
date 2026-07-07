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

const std::string THERMAL_STATE_FILE = "/run/xos/thermal.state";
const std::string FALLBACK_THERMAL_STATE = "/tmp/xos_thermal.state";
const std::string STATE_DIR = "/run/xos";
const std::string STATE_FILE = "/run/xos/fan.state";
const std::string FALLBACK_STATE_FILE = "/tmp/xos_fan.state";
const std::string CONFIG_FILE = "/etc/xos/fan.conf";
const std::string FALLBACK_CONFIG_FILE = "/tmp/xos_fan.conf";
const std::string LOG_DIR = "/var/log/xos";
const std::string LOG_FILE = "/var/log/xos/fan-control.log";

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

double read_cpu_temp() {
    std::string path = THERMAL_STATE_FILE;
    if (!is_dir_writable(STATE_DIR)) {
        path = FALLBACK_THERMAL_STATE;
    } else if (!fs::exists(path)) {
        path = FALLBACK_THERMAL_STATE;
    }

    std::ifstream file(path);
    if (!file.is_open()) return 40.0; // default safe temp

    std::string line;
    while (std::getline(file, line)) {
        if (line.rfind("cpu_temp=", 0) == 0) {
            try {
                return std::stod(line.substr(9));
            } catch (...) {}
        }
    }
    return 40.0;
}

void write_state(int speed_pct, int rpm) {
    std::string path = STATE_FILE;
    if (!is_dir_writable(STATE_DIR)) {
        path = FALLBACK_STATE_FILE;
    }

    std::string content =
        "fan_speed=" + std::to_string(speed_pct) + "\n" +
        "fan_rpm=" + std::to_string(rpm) + "\n";

    if (!atomic_write(path, content)) {
        log_message("ERROR: Could not write fan state file at " + path);
    }
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;

    std::error_code ec;
    if (!fs::exists(LOG_DIR)) {
        fs::create_directories(LOG_DIR, ec);
    }

    log_message("XOS Fan Control Daemon starting...");

    std::string cached_config_path = "";
    fs::file_time_type last_write_time;
    bool has_cache = false;
    bool prev_existed = false;
    int cached_manual_speed = -1;

    while (true) {
        double temp = read_cpu_temp();
        int speed_pct = 0;
        int manual_speed = -1;

        // Check if there is manual config override
        std::string config_path = CONFIG_FILE;
        if (!fs::exists(config_path)) {
            config_path = FALLBACK_CONFIG_FILE;
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
                std::ifstream config(config_path);
                if (config.is_open()) {
                    std::string line;
                    while (std::getline(config, line)) {
                        if (line.rfind("manual_speed=", 0) == 0) {
                            try {
                                manual_speed = std::stoi(line.substr(13));
                            } catch (...) {}
                        }
                    }
                    config.close();
                }
                auto current_write_time = fs::last_write_time(config_path, file_ec);
                if (!file_ec) {
                    last_write_time = current_write_time;
                }
            }
            cached_config_path = config_path;
            prev_existed = config_exists;
            has_cache = true;
            cached_manual_speed = manual_speed;
        } else {
            manual_speed = cached_manual_speed;
        }

        if (manual_speed >= 0 && manual_speed <= 100) {
            speed_pct = manual_speed;
        } else {
            // Apply fan curve
            // Temp <= 40 -> 20% min fan
            // Temp >= 80 -> 100% max fan
            // Linear in between
            if (temp <= 40.0) {
                speed_pct = 20;
            } else if (temp >= 80.0) {
                speed_pct = 100;
            } else {
                speed_pct = 20 + static_cast<int>((temp - 40.0) / 40.0 * 80.0);
            }
        }

        // Mock PWM to RPM (max 5000 RPM)
        int rpm = speed_pct * 50;

        write_state(speed_pct, rpm);

        // In a real hardware system:
        // Ex: write speed_pct to /sys/class/hwmon/hwmon0/pwm1

        std::this_thread::sleep_for(std::chrono::seconds(3));
    }

    return 0;
}
