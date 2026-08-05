# frozen_string_literal: true

require_relative "lib/anydoc/version"

Gem::Specification.new do |spec|
  spec.name = "anydoc"
  spec.version = Anydoc::VERSION
  spec.authors = ["Nick Pezza"]
  spec.email = ["pezza@hey.com"]

  spec.summary = "Convert documents to GitHub-Flavored Markdown"
  spec.description = "Ruby bindings for the anydoc Rust document converter."
  spec.homepage = "https://github.com/firecrawl/anydoc#readme"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.2.0"
  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/firecrawl/anydoc/tree/main/ruby"
  spec.metadata["changelog_uri"] = "https://github.com/firecrawl/anydoc/releases"
  spec.metadata["rubygems_mfa_required"] = "true"

  spec.files = Dir[
    "Cargo.toml",
    "LICENSE.txt",
    "README.md",
    "Rakefile",
    "ext/**/*",
    "lib/**/*",
  ].reject { |file| file.end_with?(".bundle", ".dll", ".dylib", ".so") }
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/anydoc/extconf.rb"]

  spec.add_dependency "rb_sys", "~> 0.9.128"
end
