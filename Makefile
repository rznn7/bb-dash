.PHONY: generate-api build run clean help

# Generate Bitbucket API client from OpenAPI spec
generate-api:
	@echo "Downloading Bitbucket API spec..."
	curl -s https://api.bitbucket.org/swagger.json -o bitbucket-api-oai.json
	@echo "Generating Rust client..."
	docker run --rm -v "$$(pwd)":/local \
		-u $$(id -u):$$(id -g) \
		-v "$$(pwd)":/local \
		openapitools/openapi-generator-cli generate \
		-i /local/bitbucket-api-oai.json \
		-g rust \
		-o /local/bitbucket-client \
		--additional-properties=packageName=bitbucket-client \
		--additional-properties=packageVersion=1.0.0 \
		--additional-properties=useSerdePathToError=true \
		--additional-properties=bestFitInt=true \
		--additional-properties=preferUnsignedInt=true \
		--additional-properties=reqwestDefaultFeatures=rustls-tls \
		--additional-properties=supportAsync=true \
		--additional-properties=supportMiddleware=true \
		--global-property=apis=Users:Pullrequests \
		--global-property=models \
		--global-property=supportingFiles \
		--model-name-prefix=Api
	@echo "✓ Client generated in ./bitbucket-client"

# Build the project
build:
	cargo build

# Run the project
run:
	cargo run

# Clean generated files and build artifacts
clean:
	cargo clean
	rm -rf bitbucket-client
	rm -f bitbucket-api-oai.json

# Show available commands
help:
	@echo "Available targets:"
	@echo "  make generate-api  - Generate Bitbucket API client"
	@echo "  make build         - Build the project"
	@echo "  make run           - Run the project"
	@echo "  make clean         - Clean generated files"
	@echo "  make help          - Show this help"
