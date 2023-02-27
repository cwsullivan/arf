import argparse
import zlib
import numpy as np
import reedsolo

RS = reedsolo.RSCodec(16)

def compress_and_duplicate(raw_data, n):
    compressed_data = zlib.compress(raw_data)

    # Convert the compressed binary string to bytes
    compressed_data_bytes = bytes(compressed_data)

    # Compute error correction codes for compressed data
    data_array = RS.encode(compressed_data_bytes)
    
    # Duplicate data array N times
    arf_data = data_array * n

    return arf_data


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

    # Check and correct errors in compressed data
    try:
        # Decode compressed data with Reed-Solomon code
        corrected_data, _, corrections = RS.decode(recovered_data)

        # Unzip compressed data to recover original data
        raw_data = zlib.decompress(corrected_data)

        print(f"Errors ({len(corrections)}) detected and corrected in compressed data")
    except reedsolo.ReedSolomonError as e:
        # Error could not be corrected
        print("Error detected and could not be corrected:", str(e))
        return None

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
            f.write(output_data)

    elif args.mode == 'decode':
        flip_percent = 0.30
        L = len(input_data)
        print(f'Corrupting {int(flip_percent*100)} percent ({int(flip_percent * L)} bits) of data...')
        flip_indices = np.random.rand(L) < flip_percent
        data = np.array(bytearray(input_data))
        data[flip_indices] = (np.random.rand(sum(flip_indices))*255).astype(int)
        corrupt_data = bytes(data)
        print('Done corrupting data..')

        # Recover original data and save to output file
        output_data = uncompress_and_recover(corrupt_data, args.n)
        with open(args.output_file, 'wb') as f:
            f.write(output_data)
    
