# Fonts

Font files (`.otf`, `.ttf`) are not tracked in git due to size. Download them manually:

## Required

**Fusion Pixel Font 12px Proportional — Traditional Chinese** (~4.7MB)

```bash
# Download from GitHub release
cd /tmp
gh release download 2026.02.27 --repo TakWolf/fusion-pixel-font \
  --pattern "fusion-pixel-font-12px-proportional-otf-*.zip"
unzip fusion-pixel-font-12px-proportional-otf-*.zip -d fusion-pixel-12px
cp fusion-pixel-12px/fusion-pixel-12px-proportional-zh_hant.otf \
  assets/fonts/fusion-pixel-12px-zh_hant.otf
```

License: SIL Open Font License 1.1 (see `LICENSE-FusionPixel-OFL.txt`)
Source: https://github.com/TakWolf/fusion-pixel-font
