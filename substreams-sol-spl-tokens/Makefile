ENDPOINT ?= mainnet.sol.streamingfast.io:443

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml map_mints -s 20000000 -t +100000000


.PHONY: protogen
protogen:
	substreams protogen substreams.yaml --exclude-paths="sf/substreams,google"

.PHONE: package
package: build
	substreams pack substreams.yaml