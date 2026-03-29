FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        clang \
        cmake \
        git \
        libudev-dev \
        libusb-1.0-0-dev \
        pkg-config \
        python3 \
        python3-pip \
        zsh \
    && rm -rf /var/lib/apt/lists/*

# Install oh-my-zsh for the vscode user
RUN su vscode -c "sh -c \"\$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\" \"\" --unattended"

# Set zsh as the default shell for vscode user
RUN chsh -s /usr/bin/zsh vscode
