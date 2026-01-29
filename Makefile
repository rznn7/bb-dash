.PHONY: generate-api build run clean help

generate-api:
	@echo "Downloading Bitbucket API spec..."
	curl -s https://api.bitbucket.org/swagger.json -o bitbucket-api-oai.json
	@echo "Patching API spec (adding missing query parameters)..."
	@jq '.paths["/repositories/{workspace}/{repo_slug}/pullrequests"].get.parameters += [{"name": "q", "in": "query", "description": "Query string to narrow down the response as per [filtering and sorting](/cloud/bitbucket/rest/intro/#filtering).", "required": false, "type": "string"}]' bitbucket-api-oai.json > bitbucket-api-oai.json.tmp && mv bitbucket-api-oai.json.tmp bitbucket-api-oai.json
	@echo "Generating Rust client..."
	docker run --rm -v "$$(pwd)":/local \
		-u $$(id -u):$$(id -g) \
		-v "$$(pwd)":/local \
		openapitools/openapi-generator-cli generate \
		-i /local/bitbucket-api-oai.json \
		-g rust \
		-o /local/bitbucket-client \
		--skip-validate-spec \
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

build:
	cargo build

run:
	cargo run

clean:
	cargo clean
	rm -rf bitbucket-client
	rm -f bitbucket-api-oai.json

help:
	@echo "Available targets:"
	@echo "  make generate-api  - Generate Bitbucket API client"
	@echo "  make build         - Build the project"
	@echo "  make run           - Run the project"
	@echo "  make clean         - Clean generated files"
	@echo "  make help          - Show this help"
