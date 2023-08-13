DATA_TO_WRITE = [
    # integer_test
    3,
    0, 0, 0, 7,
    # integer_test
    3,
    0, 0, 0, 9,
    # LSM
    1,
    0x3F, 0, 0, 0, # 0.5, x
    0x3F, 0, 0, 0, # 0.5, y
    0x3F, 0, 0, 0, # 0.5, z
]
FILE_PATH = 'data.bin'

# Write above data to file
File.open(FILE_PATH, 'wb') do |file|
  file.write(DATA_TO_WRITE.pack('C*'))
end

# Output value of 0.5 in binary 32-bit float
puts [0.5].pack('f').unpack('B*')
# Output value of 0.5 in binary 64-bit float
puts [0.5].pack('d').unpack('B*')
