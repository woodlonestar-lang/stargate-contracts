.PHONY: update-abi-snapshots check-abi-snapshots

# Regenerate committed ABI metadata under abis/ (deterministic; LC_ALL=C).
update-abi-snapshots:
	@./scripts/generate_abi_metadata.sh abis

# Verify abis/ matches freshly generated metadata (no writes).
check-abi-snapshots:
	@./scripts/generate_abi_metadata.sh /tmp/stargate-abis-check
	@diff -ru abis/ /tmp/stargate-abis-check/
