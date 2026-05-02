cask "mural" do
  version "0.1.0-beta.1"
  sha256 "25d5c992a1d9c5c73697561b184ab70088ce9af8c50f17f2bf6d33423a3a19ba"

  url "https://github.com/Fiojf/mural/releases/download/v#{version}/Mural-#{version}.dmg"
  name "Mural"
  desc "Floating wallpaper picker for macOS, summoned by a hotkey"
  homepage "https://github.com/Fiojf/mural"

  depends_on macos: ">= :monterey"

  app "Mural.app"

  caveats <<~EOS
    Mural is unsigned. On first launch, macOS Gatekeeper may block it.
    Either right-click the app and choose "Open", or run:

      xattr -dr com.apple.quarantine /Applications/Mural.app
  EOS

  zap trash: [
    "~/Library/Application Support/Mural",
    "~/Library/Caches/Mural",
    "~/Library/Preferences/app.mural.desktop.plist",
  ]
end
