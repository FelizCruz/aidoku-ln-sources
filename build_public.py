import os
import subprocess
import shutil
import sys
import glob

def get_binary_path(name):
    user_profile = os.environ.get("USERPROFILE", "")
    cargo_bin_dir = os.path.join(user_profile, ".cargo", "bin")
    
    direct_path = os.path.join(cargo_bin_dir, f"{name}.exe" if sys.platform == "win32" else name)
    if os.path.exists(direct_path):
        return direct_path
    
    found = shutil.which(name)
    if found:
        return found
    
    return name

def main():
    print("1. Compiling sources to WebAssembly (wasm32-unknown-unknown)...")
    cargo_cmd = get_binary_path("cargo")
    subprocess.run([cargo_cmd, "build", "--target", "wasm32-unknown-unknown", "--release"], check=True)

    print("\n2. Packaging .aix archives...")
    import package_aix
    package_aix.package_all()

    # Find all .aix packages
    aix_files = glob.glob("sources/*/*.aix")
    print(f"\nFound {len(aix_files)} .aix packages: {aix_files}")

    print("\n3. Building public repository with aidoku build...")
    aidoku_cmd = get_binary_path("aidoku")
    build_args = [
        aidoku_cmd, "build",
        *aix_files,
        "-o", "public",
        "--name", "Custom Light Novel Sources"
    ]
    subprocess.run(build_args, check=True)

    print("\nPublic repository ready at 'public/' folder:")
    for root, dirs, files in os.walk("public"):
        for f in sorted(files):
            rel = os.path.relpath(os.path.join(root, f), "public")
            print(f"  - public/{rel}")

if __name__ == "__main__":
    main()
