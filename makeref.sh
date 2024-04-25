cargo build
cd tests
export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib:$LD_LIBRARY_PATH
python3 run.py gen
sed -i 's/\/media\/psf\/SSD\/rust-mir-checker/\/home\/runner\/work\/rust-mir-checker\/rust-mir-checker/g' ref.results