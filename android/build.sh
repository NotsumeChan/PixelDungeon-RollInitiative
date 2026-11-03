#!/usr/bin/env bash
# APK firmado sin Gradle: cargo-ndk compila libapp.so, aapt2 arma el APK con el manifiesto, zipalign + apksigner
# lo alinean y firman. Salida: target/android/pdri.apk
#
# Variables (todas opcionales):
#   ANDROID_HOME      SDK de Android (build-tools + platforms)
#   ANDROID_NDK_HOME  NDK
#   PDRI_KEYS         keystore.properties (storeFile, storePassword, keyAlias, keyPassword); ver README
set -euo pipefail
cd "$(dirname "$0")/.."

SDK=${ANDROID_HOME:-$HOME/.buildozer/android/platform/android-sdk}
export ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-$(ls -d "$HOME"/.buildozer/android/platform/android-ndk-r*/ | sort -V | tail -1)}
BT=$(ls -d "$SDK"/build-tools/* | sort -V | tail -1)
JAR=$(ls -d "$SDK"/platforms/android-* | sort -V | tail -1)/android.jar
KEYS=${PDRI_KEYS:-$HOME/.pddnd-signing/keystore.properties}

prop() { grep "^$1=" "$KEYS" | cut -d= -f2-; }

OUT=target/android
rm -rf "$OUT"
mkdir -p "$OUT/apk/lib"
cargo ndk -t arm64-v8a -o "$OUT/apk/lib" build --release -p app --lib
"$BT/aapt2" link -o "$OUT/unsigned.apk" --manifest android/AndroidManifest.xml -I "$JAR"
(cd "$OUT/apk" && zip -qr ../unsigned.apk lib)
"$BT/zipalign" -f -p 4 "$OUT/unsigned.apk" "$OUT/aligned.apk"
PDRI_PASS=$(prop storePassword) "$BT/apksigner" sign \
    --ks "$(prop storeFile)" --ks-key-alias "$(prop keyAlias)" --ks-pass env:PDRI_PASS \
    --out "$OUT/pdri.apk" "$OUT/aligned.apk"
echo "$OUT/pdri.apk"
