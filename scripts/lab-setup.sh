#!/bin/bash

set -e

echo "=== xsshend Lab Setup ==="

# Create necessary directories
mkdir -p ssh_keys

# Generate RSA key (without passphrase) - this will be registered on targets
echo "Generating RSA key (no passphrase)..."
ssh-keygen -t rsa -b 4096 -f ssh_keys/id_rsa -N "" -C "xsshend_rsa_key"

# Generate ED25519 key (with passphrase) - this will NOT be registered on targets
echo "Generating ED25519 key (with passphrase)..."
ssh-keygen -t ed25519 -f ssh_keys/id_ed25519 -N "testpassphrase" -C "xsshend_ed25519_key"

# Create authorized_keys file with only the RSA public key
echo "Creating authorized_keys file..."
cp ssh_keys/id_rsa.pub authorized_keys
chmod 600 authorized_keys

# Set proper permissions on SSH keys
chmod 600 ssh_keys/id_rsa
chmod 600 ssh_keys/id_ed25519
chmod 644 ssh_keys/id_rsa.pub
chmod 644 ssh_keys/id_ed25519.pub

# Create hosts.json configuration file
echo "Creating hosts.json configuration..."
cat > ssh_keys/hosts.json << 'EOF'
{
  "Test": {
    "Lab": {
      "RSA-Targets": {
        "TARGET1": {
          "alias": "testuser@target1",
          "env": "TEST"
        },
        "TARGET2": {
          "alias": "testuser@target2",
          "env": "TEST"
        }
      },
      "ED25519-Targets": {
        "TARGET1_ED25519": {
          "alias": "testuser@target1",
          "env": "TEST"
        }
      }
    }
  }
}
EOF

chmod 644 ssh_keys/hosts.json

# Ensure the ssh_keys directory has proper ownership for container user (UID 1000)
echo "Setting proper ownership for ssh_keys directory..."
# Note: The container user 'master' will have UID 1000

# Create Dockerfiles
echo "Creating Dockerfile.master..."
cat > Dockerfile.master << 'EOF'
FROM archlinux:latest

RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm rust cargo git openssh base-devel && \
    pacman -Scc --noconfirm

RUN useradd -m -s /bin/bash master && \
    echo "master:masterpass" | chpasswd

USER master
WORKDIR /home/master

RUN cargo install xsshend

ENV PATH="/home/master/.cargo/bin:${PATH}"

CMD ["/bin/bash", "-c", "tail -f /dev/null"]
EOF

echo "Creating Dockerfile.target..."
cat > Dockerfile.target << 'EOF'
FROM archlinux:latest

RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm openssh sudo inetutils coreutils && \
    pacman -Scc --noconfirm

RUN useradd -m -s /bin/bash testuser && \
    echo "testuser:testpass" | chpasswd && \
    mkdir -p /home/testuser/.ssh && \
    chmod 700 /home/testuser/.ssh && \
    chown -R testuser:testuser /home/testuser/.ssh

RUN ssh-keygen -A && \
    sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin no/' /etc/ssh/sshd_config && \
    sed -i 's/#PubkeyAuthentication yes/PubkeyAuthentication yes/' /etc/ssh/sshd_config && \
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config

EXPOSE 22

CMD ["/usr/sbin/sshd", "-D"]
EOF

echo "Creating docker-compose.yml..."
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  master:
    build:
      context: .
      dockerfile: Dockerfile.master
    container_name: xsshend_master
    hostname: master
    networks:
      - xsshend_net
    volumes:
      - ./ssh_keys:/home/master/.ssh
      - master_home:/home/master
    tty: true
    stdin_open: true

  target1:
    build:
      context: .
      dockerfile: Dockerfile.target
    container_name: xsshend_target1
    hostname: target1
    networks:
      - xsshend_net
    volumes:
      - ./authorized_keys:/home/testuser/.ssh/authorized_keys:ro
    ports:
      - "2221:22"

  target2:
    build:
      context: .
      dockerfile: Dockerfile.target
    container_name: xsshend_target2
    hostname: target2
    networks:
      - xsshend_net
    volumes:
      - ./authorized_keys:/home/testuser/.ssh/authorized_keys:ro
    ports:
      - "2222:22"

networks:
  xsshend_net:
    driver: bridge

volumes:
  master_home:
EOF

echo ""
echo "=== Setup Complete ==="
echo ""
echo "SSH Keys created:"
echo "  - RSA key (NO passphrase): ssh_keys/id_rsa"
echo "  - ED25519 key (WITH passphrase 'testpassphrase'): ssh_keys/id_ed25519"
echo ""
echo "Configuration file created:"
echo "  - hosts.json: ssh_keys/hosts.json"
echo ""
echo "The RSA key is registered on target1 and target2"
echo "The ED25519 key is NOT registered on any target"
echo ""
echo "Next steps:"
echo "  1. Run: docker-compose up -d --build"
echo "  2. Wait for containers to build and start"
echo "  3. Access master: docker exec -it xsshend_master bash"
echo ""