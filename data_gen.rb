DATA_TO_WRITE = [
    3, 0, 0, 0, 7, 3, 0, 0, 0, 9
]
FILE_PATH = 'data.bin'

# Write above data to file
File.open(FILE_PATH, 'wb') do |file|
  file.write(DATA_TO_WRITE.pack('C*'))
end

# Output value of 0.5 in binary 32-bit float
puts [0.5].pack('f')