import sys
import json
from typing import Any

EXCLUDED = ['.ttc', '.ttf', '.png', '.gif', '.jpg', '.jpeg', '.bmp', '.ico', '.svg', '.webp', '.avif', '.mp4', '.avi', '.mkv', '.mp3', '.wav', '.flac', 'package-lock.json', '.lock']

def parse_wc_output(text: str) -> dict[str, Any]:
    result = {}
    total = 0

    for line in text.splitlines():
        parts = line.split()
        loc, file = int(parts[0]), parts[1]

        if any(file.endswith(ext) for ext in EXCLUDED):
            continue

        result[file] = loc
        total += loc

    files = result.copy()

    for file, loc in files.items():
        dir_path = file.rsplit('/', 1)[0] if '/' in file else ''

        while dir_path:
            result[dir_path] = result.get(dir_path, 0) + loc
            dir_path = dir_path.rsplit('/', 1)[0] if '/' in dir_path else ''

    result['total'] = total

    return result

if __name__ == "__main__":
    aliases = json.loads(sys.argv[2])

    data = sys.stdin.read().strip()
    result = parse_wc_output(data.rsplit('\n', 1)[0])

    result["ALIASES"] = {}

    for alias, path in aliases.items():
        if isinstance(path, str):
            if path in result:
                result["ALIASES"][alias] = result[path]
        elif isinstance(path, list):
            for p in path:
                if p in result:
                    result["ALIASES"][alias] = result["ALIASES"].get(alias, 0) + result[p]

    print(json.dumps(result, indent=4))
