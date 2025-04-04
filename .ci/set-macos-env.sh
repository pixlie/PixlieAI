echo "export DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/llvm/lib" >> $GITHUB_ENV
echo "export DYLD_LIBRARY_PATH=/opt/homebrew/opt/llvm/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" >> $GITHUB_ENV
echo "export PATH=/opt/homebrew/opt/llvm/bin:$PATH" >> $GITHUB_ENV
echo "export LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib" >> $GITHUB_ENV
echo "export CC=/opt/homebrew/opt/llvm/bin/clang" >> $GITHUB_ENV
echo "export CXX=/opt/homebrew/opt/llvm/bin/clang++" >> $GITHUB_ENV
echo "export ROCKSDB_INCLUDE_DIR=/opt/homebrew/opt/rocksdb/include" >> $GITHUB_ENV
echo "export ROCKSDB_LIB_DIR=/opt/homebrew/opt/rocksdb/lib" >> $GITHUB_ENV
echo "export LD=/opt/homebrew/opt/lld/bin/ld64.lld" >> $GITHUB_ENV
echo "export LDFLAGS=-fuse-ld=/opt/homebrew/opt/lld/bin/ld64.lld" >> $GITHUB_ENV
echo "export AR=/opt/homebrew/opt/llvm/bin/llvm-ar" >> $GITHUB_ENV

# TODO: For some reason, the libclang.dylib is not found by the linker.
# So we create a symlink to the libclang.dylib in /usr/local/lib
# This is a hack. There should be a better way to do this.
sudo mkdir -pv /usr/local/lib
sudo ln -s /opt/homebrew/opt/llvm/lib/libclang.dylib /usr/local/lib/libclang.dylib