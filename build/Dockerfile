FROM archlinux:latest

RUN pacman -Syu --noconfirm \
    archiso \
    git \
    base-devel \
    imagemagick \
    && pacman -Scc --noconfirm

WORKDIR /xos

COPY . /xos/

RUN chmod +x /xos/build/*.sh /xos/base/profiledef.sh /xos/base/airootfs/root/.automated_script.sh \
    /xos/base/airootfs/usr/local/bin/xos-first-login 2>/dev/null || true

CMD ["/xos/build/build-iso.sh"]
