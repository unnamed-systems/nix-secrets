const referencePages = [
  "generators_reference",
  "nixos-configuration-options",
  "home-manager-configuration-options",
  "hjem-configuration-options",
  "nix-darwin-configuration-options"
];

if (referencePages.some(page => window.location.pathname.includes(page))) {
  const button = document.querySelector("#git-edit-button");

  if (button) {
    button.style.display = "none";
  }
}