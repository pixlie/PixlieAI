# Backup API Types
[ -d admin/src/api_types_bak ] && rm -rf admin/src/api_types_bak
mv admin/src/api_types admin/src/api_types_bak

# Generate API Types
export TS_RS_EXPORT_DIR="../admin/src/api_types"
cd pixlie_ai
cargo test || \
    (mv ../admin/src/api_types_bak ../admin/src/api_types && echo "\nerror: Error generating types, reverted all changes." && exit 1)

# Format API Types
cd ../admin
(pnpm prettier src/api_types --write && rm -rf src/api_types_bak) ||
    (rm -rf src/api_types && mv src/api_types_bak src/api_types && echo "\nError formatting types, reverted all changes." && exit 1)
