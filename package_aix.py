import os
import zipfile
import shutil

def build_aix(source_dir, output_aix):
    res_dir = os.path.join(source_dir, "res")
    wasm_file = os.path.join("target", "wasm32-unknown-unknown", "release", "wetriedtls.wasm")
    
    if not os.path.exists(wasm_file):
        raise FileNotFoundError(f"WASM file not found at {wasm_file}. Please run cargo build first.")
    
    if not os.path.exists(res_dir):
        raise FileNotFoundError(f"res directory not found at {res_dir}")

    os.makedirs(os.path.dirname(output_aix) if os.path.dirname(output_aix) else ".", exist_ok=True)
    
    with zipfile.ZipFile(output_aix, "w", zipfile.ZIP_DEFLATED) as zf:
        # Add main.wasm
        zf.write(wasm_file, arcname="Payload/main.wasm")
        print(f"Added Payload/main.wasm ({os.path.getsize(wasm_file)} bytes)")
        
        # Add res directory files
        for fn in os.listdir(res_dir):
            full_p = os.path.join(res_dir, fn)
            if os.path.isfile(full_p):
                zf.write(full_p, arcname=f"Payload/{fn}")
                print(f"Added Payload/{fn} ({os.path.getsize(full_p)} bytes)")

    print(f"Successfully packaged -> {output_aix} ({os.path.getsize(output_aix)} bytes)")

if __name__ == "__main__":
    build_aix("sources/wetriedtls", "sources/wetriedtls/wetriedtls.aix")
