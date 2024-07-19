import json
import os

download_iso = os.environ.get("download_iso")
edition_iso = os.environ.get("edition_iso")

input_file = f"data/kaikki/{edition_iso}-extract.jsonl"
output_file = f"data/kaikki/{download_iso}-{edition_iso}-extract.jsonl"

print(f"Reading {input_file} and writing {output_file}...")

with open(input_file, "r", encoding="utf-8") as input_file, \
     open(output_file, "w", encoding="utf-8") as output_file:

    line_count = 0
    print_interval = 1000

    for line in input_file:
        line_count += 1

        try:
            obj = json.loads(line.strip())
        except json.JSONDecodeError:
            print(f"Error decoding JSON in line {line_count}. Skipping...")
            continue

        if "lang_code" not in obj:
            if "redirect" not in obj:
                print(f"Error: no lang_code or redirect in line {line_count}.", obj)
            continue

        if obj["lang_code"] == download_iso:
            output_file.write(line)

        # Print progress at the specified interval
        if line_count % print_interval == 0:
            print(f"Processed {line_count} lines...", end="\r")

print("\nFinished.")
