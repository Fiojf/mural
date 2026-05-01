cask "mural" do
  version "0.1.0"
  sha256 :no_check

  url "https://github.com/Fiojf/mural/releases/download/v#{version}/Mural-#{version}.dmg"
  name "Mural"
  desc "Floating wallpaper picker for macOS"
  homepage "https://github.com/Fiojf/mural"

  depends_on macos: ">= :monterey"

  app "Mural.app"

  caveats <<~EOS
    Mural is currently unsigned. On first launch, macOS Gatekeeper may block it.
    Either right-click the app and choose "Open", or run:

      xattr -d com.apple.quarantine /Applications/Mural.app
  EOS

  zap trash: [
    "~/Library/Application Support/Mural",
    "~/Library/Caches/Mural",
    "~/Library/Preferences/app.mural.desktop.plist",
  ]
end
