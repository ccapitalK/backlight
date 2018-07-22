#!/usr/bin/bash

BIN_BUILD_PATH=target/release/backlight
INSTALL_PATH=/usr/bin/backlight

cargo build --release || exit 1
echo installing
sudo cp $BIN_BUILD_PATH $INSTALL_PATH || exit 1
sudo chown root $INSTALL_PATH 
sudo chgrp root $INSTALL_PATH 
sudo chmod 4755 $INSTALL_PATH 
