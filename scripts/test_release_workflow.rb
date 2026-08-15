#!/usr/bin/env ruby
# frozen_string_literal: true

workflow = File.read(
  File.expand_path("../.github/workflows/release.yml", __dir__),
  encoding: "UTF-8"
)

required_fragments = [
  'tags: ["v*.*.*"]',
  "cargo-cyclonedx --version 0.5.9 --locked",
  "recoverable-delete-${RELEASE_TAG}.cdx.json",
  'normalized_ref="pkg:cargo/recoverable-delete@${RELEASE_TAG#v}"',
  ".metadata.component[\"bom-ref\"] = $normalized_ref",
  "if grep -Eq 'path\\+file:|download_url=file:'",
  "actions/attest-build-provenance@",
  "actions/attest@",
  "sbom-path:",
  "name=\"Recoverable Delete ${RELEASE_TAG}\"",
  "release_id=\"$(gh api"
]

required_fragments.each do |fragment|
  abort("release workflow is missing #{fragment.inspect}") unless workflow.include?(fragment)
end

abort("release workflow must publish through gh, not a release action") if workflow.include?("softprops/action-gh-release")

action_references = workflow.scan(/^\s*uses:\s*([^@\s]+)@([^\s#]+)/).map do |name, revision|
  [name, revision]
end
unpinned_actions = action_references.reject { |_name, revision| revision.match?(/\A[0-9a-f]{40}\z/) }
unless unpinned_actions.empty?
  abort("release workflow has unpinned actions: #{unpinned_actions.map(&:first).join(', ')}")
end

puts "release workflow contract passed"
