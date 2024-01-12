import json
import seaborn as sns
import matplotlib.pyplot as plt
import pandas as pd

# import os
# counter = {}
# for target_iso in ['de', 'es', 'ru', 'zh', 'fr']:
#     print(f"Processing {target_iso}...")
#     counter[target_iso] = {}
#     with open(f'../data/kaikki/{target_iso}-extract.json', "r", encoding="utf-8") as f:
#         line_count = 0
#         print_interval = 1000

#         for line in f:
#             line_count += 1
#             if line_count % print_interval == 0:
#                 print(f"Processed {line_count} lines...", end='\r')
#             try:
#                 obj = json.loads(line.strip())
#             except json.JSONDecodeError:
#                 print(f"Error decoding JSON in line {line_count}. Skipping...")
#                 continue

#             if "lang_code" in obj:
#                 counter[target_iso][obj["lang_code"]] = counter[target_iso].get(obj["lang_code"], 0) + 1
#             else:
#                 if "redirect" in obj:
#                     counter[target_iso]["redirect"] = counter[target_iso].get("redirect", 0) + 1
#                 else:
#                     counter[target_iso]["error"] = counter[target_iso].get("error", 0) + 1
#     print(json.dumps(counter[target_iso], indent=4))

# print(f"Processing en...")
# counter["en"] = {}
# for file in os.listdir("../data/kaikki"):
#     if file.startswith("kaikki"):
#         print(f"Processing {file}...")
#         with open(f"../data/kaikki/{file}", "r", encoding="utf-8") as f:
#             line_count = 0
#             print_interval = 1000

#             for line in f:
#                 line_count += 1
#                 if line_count % print_interval == 0:
#                     print(f"Processed {line_count} lines...", end='\r')
#                 try:
#                     obj = json.loads(line.strip())
#                 except json.JSONDecodeError:
#                     print(f"Error decoding JSON in line {line_count}. Skipping...")
#                     continue

#                 if "lang_code" in obj:
#                     counter["en"][obj["lang_code"]] = counter["en"].get(obj["lang_code"], 0) + 1
#                 else:
#                     if "redirect" in obj:
#                         counter["en"]["redirect"] = counter["en"].get("redirect", 0) + 1
#                     else:
#                         counter["en"]["error"] = counter["en"].get("error", 0) + 1

# for target_iso in counter:
#     for target_iso2 in counter:
#         for source_iso in counter[target_iso]:
#             if source_iso not in counter[target_iso2]:
#                 counter[target_iso2][source_iso] = 0

# for target_iso in counter:
#     if "error" in counter[target_iso]:
#         del counter[target_iso]["error"]
#     if "redirect" in counter[target_iso]:
#         del counter[target_iso]["redirect"]
#     counter[target_iso] = {k: v for k, v in sorted(counter[target_iso].items(), key=lambda item: item[0])}

# heatmap_data = [[counter[key1].get(key2, 0) for key2 in counter[key1]] for key1 in counter]

# source_languages = list(counter.keys())
# target_languages = list(counter["de"].keys())

# with open('heatmap_data.json', 'w') as f:
#     json.dump(heatmap_data, f)

# with open('source_languages.json', 'w') as f:
#     json.dump(source_languages, f)

# with open('target_languages.json', 'w') as f:
#     json.dump(target_languages, f)

with open('heatmap_data.json', 'r') as f:
    heatmap_data = json.load(f)

with open('source_languages.json', 'r') as f:
    source_languages = json.load(f)

with open('target_languages.json', 'r') as f:
    target_languages = json.load(f)

annotations = []
for row in heatmap_data:
    new_row = []
    for cell in row:
        if cell < 1000:
            new_row.append(str(cell))
        else:
            rounded_value = int(round(cell / 1000, 0))
            new_row.append(f"{rounded_value}k")
    annotations.append(new_row)

size = 25

df = pd.DataFrame(heatmap_data, index=source_languages, columns=target_languages)
df = df.loc[df.sum(axis=1).sort_values(ascending=False).head(size).index]
df = df[df.sum().sort_values(ascending=False).head(size).index]

annotations = pd.DataFrame(annotations, index=source_languages, columns=target_languages)
annotations = annotations.loc[df.sum(axis=1).sort_values(ascending=False).head(size).index]
annotations = annotations[df.sum().sort_values(ascending=False).head(size).index]

# Set a larger figure size
plt.figure(figsize=(15, 4))

# Create a heatmap using seaborn
sns.heatmap(df, annot=annotations, cmap="YlGnBu", annot_kws={"size": 7}, fmt="s", vmax=150000, cbar_kws={'label': 'number of words'})

# Add labels and title
plt.xlabel("Source Language (headwords in this language)", fontsize=8)
plt.ylabel("Target Language (glosses in this language)", fontsize=8)
plt.title("yzkW", fontsize=12)

# Save the plot with a higher resolution
plt.savefig("heatmap.png", dpi=300)
