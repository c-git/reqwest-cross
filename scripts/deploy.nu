#!/usr/bin/env nu

# Tags the most recent commit as a new release. Fails if the version has not been bumped because the tag will already exist
def main [] {
    # Ensure the working tree is clean
    if ((git status --porcelain | str trim) != "") {
        print "Uncommitted changes detected. Exiting script."
        exit 1
    }

    # Check if Cargo.toml exists
    if not ("Cargo.toml" | path exists) {
        print ("Error: Cargo.toml not found in the current directory.")
        exit 1
    }

    # Open and parse the TOML file, then extract the version
    let cargo_toml_contents = open Cargo.toml
    let version = ($cargo_toml_contents | get package.version)
    let crate_name = ($cargo_toml_contents | get package.name)
    let tag_name = $"($crate_name)_v($version)"

    # Ensure not a dev version
    if "dev" in $tag_name {
        print $"Error: Current version is a development version. NOT tagged: ($tag_name)"
        exit 1
    }
    
    # Ensure we are on the main branch
    let current_branch = (git branch --show-current | str trim)

    if $current_branch != "main" {
        print $"Error: You are on branch '($current_branch)', not 'main'."
        exit 1
    }
    
    # Ensure cargo-semver-checks passes
    cargo semver-check

    git push
    git tag $tag_name
    git push --tags
    cargo publish
    print $"Tag ($tag_name) created successfully and pushed"    
}