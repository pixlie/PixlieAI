export TS_RS_EXPORT_DIR="../admin/src/api_types"
cd pixlieai_rs && \
    cargo test && \
    cd ..

[ -e admin/src/api_types ] && rm -rf src/admin/api_types
cd admin &&
    pnpm prettier src/api_types --write
