import urllib.request
import re
import json

doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
url = f"https://docs.google.com/document/d/{doc_id}/edit?usp=sharing"

req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'})
html = urllib.request.urlopen(req).read().decode('utf-8', errors='ignore')

# Test string scanner for text and style ops
def scan_text_and_styles(html):
    # 1. Extract text from "s":"..."
    text_chunks = []
    pos = 0
    pattern = '"s":"'
    while True:
        idx = html[pos:].find(pattern)
        if idx == -1:
            break
        abs_start = pos + idx + len(pattern)
        sub = html[abs_start:]
        in_escape = False
        end_idx = len(sub)
        for byte_idx, c in enumerate(sub):
            if in_escape:
                in_escape = False
            elif c == '\\':
                in_escape = True
            elif c == '"':
                end_idx = byte_idx
                break
        raw_val = sub[:end_idx]
        try:
            val = json.loads(f'"{raw_val}"')
            text_chunks.append(val)
        except Exception:
            text_chunks.append(raw_val.replace('\\n', '\n').replace('\\"', '"'))
        pos = abs_start + end_idx + 1
        
    full_text = "".join(text_chunks)
    flags = bytearray(len(full_text))
    
    # 2. Extract styles from {"ty":"as"...}
    as_pattern = '{"ty":"as"'
    pos = 0
    as_count = 0
    bold_count = 0
    italic_count = 0
    while True:
        idx = html[pos:].find(as_pattern)
        if idx == -1:
            break
        abs_start = pos + idx
        # Find closing brace of this JSON object
        # Since 'as' objects are shallow or have {sm:{...}}, find the matching closing brace
        depth = 0
        end_idx = abs_start
        while end_idx < len(html):
            if html[end_idx] == '{':
                depth += 1
            elif html[end_idx] == '}':
                depth -= 1
                if depth == 0:
                    break
            end_idx += 1
            
        obj_str = html[abs_start:end_idx+1]
        as_count += 1
        
        # Check if it's text style
        if '"st":"text"' in obj_str or "'st':'text'" in obj_str:
            is_bold = '"ts_bd":true' in obj_str or '"ts_bd": true' in obj_str
            is_italic = '"ts_it":true' in obj_str or '"ts_it": true' in obj_str
            
            if is_bold or is_italic:
                # Extract si and ei
                si_m = re.search(r'"si":\s*(\d+)', obj_str)
                ei_m = re.search(r'"ei":\s*(\d+)', obj_str)
                if si_m and ei_m:
                    si = int(si_m.group(1))
                    ei = int(ei_m.group(1))
                    start = max(0, si - 1)
                    end = min(len(full_text), ei)
                    mask = (1 if is_bold else 0) | (2 if is_italic else 0)
                    for k in range(start, end):
                        flags[k] |= mask
                    if is_bold: bold_count += 1
                    if is_italic: italic_count += 1
                    
        pos = end_idx + 1
        
    print(f"Scanned: {len(full_text)} chars, {as_count} style objects, {bold_count} bolds, {italic_count} italics")
    return full_text, flags

text, flags = scan_text_and_styles(html)
# Let's verify sample bold text
ch1_start = text.find("Chapter 01")
ch2_start = text.find("Chapter 02")
print(f"Chapter 1 slice: [{ch1_start}..{ch2_start}]")

ch1_flags = flags[ch1_start:ch2_start]
bold_chars = sum(1 for f in ch1_flags if f & 1)
italic_chars = sum(1 for f in ch1_flags if f & 2)
print(f"Chapter 1 styled characters: Bold={bold_chars}, Italic={italic_chars}")
