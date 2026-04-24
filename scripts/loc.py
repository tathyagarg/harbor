"""
This script calculates the lines of code (LOC) for a project, excluding certain file types and directories. It is intended to be used with this script in the `pre-commit` git hook:

```sh
#!/bin/sh

git ls-files \
| xargs wc -l \
| python scripts/loc.py \
    -- '{
      "css": "harbor/engine/src/css",
      "js": ["harbor/engine/src/js", "harbor/js"],
      "html": "harbor/engine/src/html5",
      "http": "harbor/engine/src/http",
      "font": "harbor/engine/src/font",
      "render": "harbor/engine/src/render"
    }' \
> .github/lines.json

echo "Updated lines.json"
git add .github/lines.json
```
"""

import sys
import json
from typing import Any

EXCLUDED = ['.ttc', '.ttf', '.png', '.gif', '.jpg', '.jpeg', '.bmp', '.ico', '.svg', '.webp', '.avif', '.mp4', '.avi', '.mkv', '.mp3', '.wav', '.flac', 'package-lock.json', '.lock']
IGNORE_DIRS = ['docs']

TOTAL_SYMBOL = "__total__"

def sum_loc(data: dict[str, Any]) -> int:
    total = 0

    for key, value in data.items():
        if key == 'total':
            continue
        elif isinstance(value, dict):
            total += sum_loc(value)
        elif isinstance(value, int):
            total += value

    return total

def complete_sum_loc(data: dict[str, Any]) -> dict[str, Any]:
    result = {}

    for key, value in data.items():
        if key == TOTAL_SYMBOL:
            continue
        elif isinstance(value, dict):
            result[key] = complete_sum_loc(value)
            result[key][TOTAL_SYMBOL] = sum_loc(result[key])
        elif isinstance(value, int):
            result[key] = value

    return result

def parse_wc_output(text: str) -> tuple[dict[str, Any], int]:
    result = {}
    total_loc = 0
    file_count = 0

    for fdata in text.splitlines():
        parts = fdata.split()
        loc, fname = int(parts[0]), parts[1]

        if any(fname.endswith(ext) for ext in EXCLUDED):
            continue

        place_location = result
        place = True

        while '/' in fname:
            dir_name, fname = fname.split('/', 1)
            if dir_name in IGNORE_DIRS:
                place = False
                break

            place_location = place_location.setdefault(dir_name, {TOTAL_SYMBOL: 0})

        if place:
            place_location[fname] = loc
            total_loc += loc
            file_count += 1

    complete_result = complete_sum_loc(result)
    complete_result[TOTAL_SYMBOL] = total_loc

    return (complete_result, file_count)

if __name__ == "__main__":
    data = sys.stdin.read().strip()
    result, fc = parse_wc_output(data.rsplit('\n', 1)[0])

    data = {
        'lines': result,
        'file_count': fc
    }

    print(json.dumps(data, indent=4))
