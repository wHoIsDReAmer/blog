# Makefile for Zola blog

# Default Zola project directory
ZOLA_DIR = my_blog
OUTPUT_DIR = $(ZOLA_DIR)/public

.PHONY: help serve build check clean test_local

help:
	@echo "Available commands:"
	@echo "  make serve        - Start the local Zola development server"
	@echo "  make build        - Build the Zola site for production"
	@echo "  make check        - Check the built site for issues (e.g., broken links)"
	@echo "  make clean        - Remove the public build directory"
	@echo "  make test_local   - Clean, build, and check the site locally"

serve:
	@cd $(ZOLA_DIR) && zola serve

build:
	@cd $(ZOLA_DIR) && zola build

check: build
	@cd $(ZOLA_DIR) && zola check

clean:
	@rm -rf $(OUTPUT_DIR)
	@echo "Cleaned $(OUTPUT_DIR)"

test_local: clean build check
	@echo "Local test completed. Site built in $(OUTPUT_DIR) and checked."
