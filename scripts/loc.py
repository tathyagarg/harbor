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

def parse_wc_output(text: str) -> tuple[dict[str, Any], int]:
    result = {}
    total = 0

    file_count = 0

    for line in text.splitlines():
        parts = line.split()
        loc, file = int(parts[0]), parts[1]

        if any(file.endswith(ext) for ext in EXCLUDED):
            continue

        result[file] = loc
        total += loc

        file_count += 1

    files = result.copy()

    for file, loc in files.items():
        dir_path = file.rsplit('/', 1)[0] if '/' in file else ''

        while dir_path:
            result[dir_path] = result.get(dir_path, 0) + loc
            dir_path = dir_path.rsplit('/', 1)[0] if '/' in dir_path else ''

    result['total'] = total

    return (result, file_count)

if __name__ == "__main__":
    aliases = json.loads(sys.argv[2])

    data = sys.stdin.read().strip()
    result, fc = parse_wc_output(data.rsplit('\n', 1)[0])

    result["ALIASES"] = {}

    for alias, path in aliases.items():
        if isinstance(path, str):
            if path in result:
                result["ALIASES"][alias] = result[path]
        elif isinstance(path, list):
            for p in path:
                if p in result:
                    result["ALIASES"][alias] = result["ALIASES"].get(alias, 0) + result[p]

    data = {
        'lines': result,
        'file_count': fc
    }

    print(json.dumps(data, indent=4))
