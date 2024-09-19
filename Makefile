# Variables
RPMBUILD_DIR ?= $(HOME)/rpmbuild
SRC_DIR      ?= tmp/rat-0.1.0
SRC_ARCHIVE  ?= $(RPMBUILD_DIR)/SOURCES/rat-0.1.0.tar.gz
SPEC_FILE    ?= rat.spec
PACKAGE_NAME ?= rat
VERSION      ?= 0.1.0
ARCH         ?= x86_64

# Files to be cleaned
CLEAN_FILES  = fx fxy fy in in2 in2b out out1 a largefile.txt

# Clean target to remove generated files
clean:
	rm -f $(CLEAN_FILES)

# RPM build process
rpm: clean
	# Remove old build directories
	rm -rf $(RPMBUILD_DIR)
	
	# Create necessary RPM build directories
	mkdir -p $(RPMBUILD_DIR)/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	
	# Copy spec file to RPM SPECS directory
	cp $(SPEC_FILE) $(RPMBUILD_DIR)/SPECS/
	
	# Prepare source directory and archive
	mkdir -p $(SRC_DIR)
	cp -r rat tests build.rs Cargo.lock Cargo.toml LICENSE README.md README.en.md $(SRC_DIR)/
	cd tmp && tar czvf $(SRC_ARCHIVE) rat-$(VERSION)
	rm -rf tmp
	
	# Build RPM package
	rpmbuild -ba $(RPMBUILD_DIR)/SPECS/$(SPEC_FILE)

# Install the built RPM
rpm-install:
	rpm -ivh $(RPMBUILD_DIR)/RPMS/$(ARCH)/$(PACKAGE_NAME)-$(VERSION)-1.*$(ARCH).rpm

# Uninstall the RPM package
rpm-uninstall:
	rpm -e $(PACKAGE_NAME)

# Phony targets
.PHONY: all clean rpm rpm-install rpm-uninstall
