# Backup API Types
[ -e admin/src/api_types_bak ] && rm -r admin/src/api_types_bak
mv admin/src/api_types admin/src/api_types_bak

# # Generate API Types
export TS_RS_EXPORT_DIR="../admin/src/api_types"
cd pixlie_ai && \
    if [[ " $@ " =~ " --verbose " ]]; then
        cargo test --verbose
    else
        cargo test
    fi && \
    cd .. || (cd .. && mv admin/src/api_types_bak admin/src/api_types && exit 1)

# # Format API Types
cd admin &&
    (pnpm prettier src/api_types --write && rm -r src/api_types_bak) ||
        (rm -r src/api_types && mv src/api_types_bak src/api_types && exit 1)
