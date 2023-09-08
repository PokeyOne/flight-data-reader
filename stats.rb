#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"
require "tempfile"

# This is a simple script to count the number of lines in all Rust files in the
# current repository. It is not meant to be super efficient, and is mostly
# cobbled together with the help of GitHub CoPilot.

# get all rs files and calculate line count
def get_rs_files
  Dir.glob("**/*.rs")
end

def calculate_file_line_count(file)
  line_count = 0
  char_count = 0
  File.open(file, "r") do |f|
    f.each_line do |line|
      line_count += 1
      char_count += line.length
    end
  end
  { line_count: line_count, char_count: char_count }
end

total_count = 0
total_char_count = 0
entries = []
get_rs_files.each do |file|
  next if file.include?("target")

  calculated_counts = calculate_file_line_count(file)
  intermediate_count = calculated_counts[:line_count]
  intermediate_char_count = calculated_counts[:char_count]
  total_count += intermediate_count
  total_char_count += intermediate_char_count
  entries << { file: file, line_count: intermediate_count, char_count: intermediate_char_count }
  puts "#{file} #{intermediate_count} (#{intermediate_char_count} chars)"
end

puts "Total: #{total_count} (#{total_char_count} chars)"
