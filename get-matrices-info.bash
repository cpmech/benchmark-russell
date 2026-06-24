#!/bin/bash

# Define a permanent local filename for the master index
mkdir -p data
INDEX_FILE="data/sparse_index.html"
OUTPUT_FILE="data/matrices.json"

# Verify jq is installed before starting
if ! command -v jq &> /dev/null; then
    echo "Error: 'jq' is required but not installed. Run 'sudo apt install jq'."
    exit 1
fi

MATS="
    bwm2000 \
    rdb5000 \
    Goodwin_040 \
    fp \
    xenon1 \
    twotone \
    Raj1 \
    boyd2 \
    Goodwin_071 \
    darcy003 \
    rma10 \
    helm2d03 \
    stomach \
    oilpan \
    ASIC_680k \
    tmt_unsym \
    Goodwin_127 \
    pre2 \
    marine1 \
    torso1 \
    atmosmodd \
    atmosmodl \
    memchip \
    Freescale1 \
    rajat31 \
    Transport \
    inline_1 \
    PFlow_742 \
    Emilia_923 \
    dielFilterV2real \
    Flan_1565 \
    mhd1280b \
    mplate \
    RFdevice \
    vfem \
    fem_filter \
    Chevron4 \
    mono_500Hz \
    kim2 \
    fem_hifreq_circuit \
    dielFilterV3clx \
"

if [ -f "$INDEX_FILE" ]; then
    echo "Using cached local index: $INDEX_FILE"
else
    echo "Local index not found. Downloading SuiteSparse master index..."
    curl -s "https://sparse.tamu.edu/?per_page=All" > "$INDEX_FILE"
fi

# Reset/Initialize the JSON file as an empty array
echo "[]" > "$OUTPUT_FILE"

# Highly stable layout-agnostic field extractor via Perl slurping
extract_field() {
    local html="$1"
    local label="$2"
    echo "$html" | perl -0777 -ne '
        if (/<th[^>]*>\s*\Q'"${label}"'\E.*?<\/th>\s*<td[^>]*>(.*?)<\/td>/s) {
            my $val = $1;
            $val =~ s/<[^>]*>//g;
            $val =~ s/^\s+|\s+$//g;
            print $val;
        }
    '
}

for mat in $MATS; do
    echo "Processing: $mat..."

    # Extract the relative path robustly regardless of full URL prefixes or quote types
    rel_path=$(grep -oP 'href=["'\''][^"'\'']*/'"${mat}"'["'\'']' "$INDEX_FILE" | head -n 1 | sed -E 's|href=["'\'']||; s|["'\'']$||; s|https://sparse.tamu.edu/||; s|^/||' || true)

    if [ -z "$rel_path" ]; then
        echo "   [!] Error: Could not locate path for '$mat' in index file."
        continue
    fi

    group=$(echo "$rel_path" | cut -d'/' -f1)
    url="https://sparse.tamu.edu/${rel_path}"

    # Fetch the target webpage contents
    page_html=$(curl -s "$url")

    # Cleanly pull the metrics
    num_rows=$(extract_field "$page_html" "Num Rows")
    num_cols=$(extract_field "$page_html" "Num Cols")
    nonzeros=$(extract_field "$page_html" "Nonzeros")
    pattern_entries=$(extract_field "$page_html" "Pattern Entries")
    kind=$(extract_field "$page_html" "Kind")
    symmetric=$(extract_field "$page_html" "Symmetric")
    pos_def=$(extract_field "$page_html" "Positive Definite")
    data_type=$(extract_field "$page_html" "Type")

    # Isolate the plain-text description from the <div class='h4'> element
    
    description=$(echo "$page_html" | perl -0777 -ne '
        if (/\s*<\/h[12]>\s*.*?<div\s+class=.h4.>(.*?)<\/div>/s) {
            my $d = $1;
            $d =~ s/<[^>]*>//g;
            $d =~ s/^\s+|\s+$//g;
            print $d;
        }
    ' || true)

    # Append safely to JSON using native jq parsing
    jq --arg name "$mat" \
       --arg group "$group" \
       --arg rows "$num_rows" \
       --arg cols "$num_cols" \
       --arg nz "$nonzeros" \
       --arg pe "$pattern_entries" \
       --arg kind "$kind" \
       --arg sym "$symmetric" \
       --arg pd "$pos_def" \
       --arg type "$data_type" \
       --arg desc "$description" \
       '. += [{
           "Name": $name,
           "Group": $group,
           "Num Rows": ($rows | gsub(","; "") | tonumber? // null),
           "Num Cols": ($cols | gsub(","; "") | tonumber? // null),
           "Nonzeros": ($nz | gsub(","; "") | tonumber? // null),
           "Pattern Entries": ($pe | gsub(","; "") | tonumber? // null),
           "Kind": $kind,
           "Symmetric": $sym,
           "Positive Definite": (if $pd == "" then null else $pd end),
           "Type": $type,
           "Description": (if $desc == "" then null else $desc end)
       }]' "$OUTPUT_FILE" > temp.json && mv temp.json "$OUTPUT_FILE"

    # Wait a little before next matrix
    sleep 0.5 # half a second
done

echo "-----------------------------------"
echo "Done! Data safely processed into $OUTPUT_FILE"
