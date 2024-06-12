cp config.toml /root/home/rust
cd /root/home/rust
./x.py build --stage 2 -j$(nproc)
./x.py dist -j$(nproc)
./x.py install -j$(nproc)