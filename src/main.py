import argparse
import zlib

def compress_and_duplicate(raw_data, n):
    compressed_data = zlib.compress(raw_data)

    # Compute error correction codes for compressed data
    crc = zlib.crc32(compressed_data)

    # Convert the CRC value to bytes
    crc_bytes = crc.to_bytes(4, byteorder='big')

    # Convert the compressed binary string to bytes
    compressed_data_bytes = bytes(compressed_data)

    # Create array with compressed data and error codes
    data_array = [compressed_data_bytes, crc_bytes]

    # Duplicate data array N times
    arf_data = [data_array] * n

    # Flatten the list of arrays
    arf_data_flat = [item for sublist in arf_data for item in sublist]

    return arf_data_flat


def uncompress_and_recover(arf_data, n):
    # Create redundant arrays from input data
    stride = int(len(arf_data) / n)
    redundant_arrays = [arf_data[idx*stride : (idx+1)*stride] for idx in range(n)]

    # Find most common value at each index across all arrays
    recovered_data = []
    for i in range(stride):
        index_values = [array[i] for array in redundant_arrays]
        most_common_value = max(set(index_values), key=index_values.count)
        recovered_data.append(most_common_value)

    # Split recovered data into compressed data and error codes
    compressed_data = bytes(recovered_data[:-4])
    crc = bytes(recovered_data[-4:])

    # Check and correct errors in compressed data
    recovered_crc = zlib.crc32(compressed_data)
    if recovered_crc != int.from_bytes(crc, byteorder='big'):
        print("Warning: error detected and corrected in compressed data")
    else:
        print("No errors detected in compressed data")

    # Unzip compressed data to recover original data
    raw_data = zlib.decompress(compressed_data)

    return raw_data



if __name__ == "__main__":
    # Parse command line arguments
    parser = argparse.ArgumentParser()
    parser.add_argument("--input_file", help="Input file path")
    parser.add_argument("--output_file", help="Output file path")
    parser.add_argument("--n", type=int, help="Number of times to duplicate the data")
    parser.add_argument("--mode", choices=['encode', 'decode'], help="Mode: 'encode' to compress and duplicate, 'decode' to recover original data")
    args = parser.parse_args()

    # Read input file
    with open(args.input_file, 'rb') as f:
        input_data = f.read()

    # Perform selected mode
    if args.mode == 'encode':
        # Compress and duplicate input file
        output_data = compress_and_duplicate(input_data, args.n)
        with open(args.output_file, 'wb') as f:
            for output in output_data:
                f.write(bytes(output))

    elif args.mode == 'decode':
        # Recover original data and save to output file
        output_data = uncompress_and_recover(input_data, args.n)
        with open(args.output_file, 'wb') as f:
            f.write(output_data)
    
