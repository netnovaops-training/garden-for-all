# A throwaway "Jun's laptop" for the installer test (wizard_test.sh): Ubuntu 24.04 with systemd as
# the first process (so services start like on a real machine), user jun with sudo. Run privileged
# (Docker runs inside it). Nothing here is published on the lab machine's network.
FROM ubuntu:24.04
ENV container=docker DEBIAN_FRONTEND=noninteractive
RUN apt-get update -qq && apt-get install -y -qq systemd systemd-sysv sudo dbus curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && (userdel -r ubuntu 2>/dev/null || true) \
    && useradd -m -s /bin/bash -c Jun jun && echo 'jun ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/jun \
    && systemctl mask systemd-logind getty.target console-getty.service
STOPSIGNAL SIGRTMIN+3
CMD ["/sbin/init"]
