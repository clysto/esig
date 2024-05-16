#!/bin/bash

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <path_to_image> <output_dir>"
    exit 1
fi

input_image=$1
base_name=$(basename "$input_image" .png)

output_dir=$2
mkdir -p $output_dir

sizes=(16 32 48 64 128 256 512)

for size in "${sizes[@]}"
do
    output_file="${output_dir}/${base_name}_${size}x${size}.png"
    convert "$input_image" -resize ${size}x${size} "$output_file"
    echo "Generated $output_file"
done

# ico_file="${output_dir}/${base_name}.ico"
# convert "${output_dir}/${base_name}"_*x*.png $ico_file
# echo "Generated ICO file at $ico_file"
