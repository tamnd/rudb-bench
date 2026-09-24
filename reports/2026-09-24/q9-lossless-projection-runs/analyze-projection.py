import collections
import mmap
import struct
import sys

path = sys.argv[1]
with open(path, 'rb') as file:
    data = mmap.mmap(file.fileno(), 0, access=mmap.ACCESS_READ)
    base = data.find(b'RUDBRP1\x00')
    if base < 0:
        raise SystemExit('projection not found')
    rows, dictionary, pages, width = struct.unpack_from('<QHIB', data, base + 12)
    header = 27 + dictionary * 4
    histogram = collections.Counter()
    page_rows_total = 0
    runs = 0
    distinct_pairs = 0
    repeated_runs = 0
    uniform_runs = 0
    for page in range(pages):
        start = base + page * 524288
        prefix = header if page == 0 else 0
        used, page_rows = struct.unpack_from('<II', data, start + prefix)
        cursor = start + prefix + 24
        end = cursor + used
        page_rows_total += page_rows
        while cursor < end:
            length = struct.unpack_from('<I', data, cursor + 8)[0]
            histogram[length] += 1
            runs += 1
            codes = data[cursor + 12:cursor + 12 + length * width]
            unique = len(set(struct.iter_unpack('<H' if width == 2 else '<B', codes)))
            distinct_pairs += unique
            repeated_runs += unique < length
            uniform_runs += unique == 1
            cursor += 12 + length * width
        if cursor != end:
            raise SystemExit(f'bad page {page}')
    print('rows', rows, 'dictionary', dictionary, 'pages', pages, 'width', width)
    print('decoded_rows', page_rows_total, 'runs', runs, 'mean_run', rows / runs)
    for length in (1, 2, 3, 4, 5):
        print('length', length, 'runs', histogram[length], 'share', histogram[length] / runs)
    print('longer_runs', sum(count for length, count in histogram.items() if length > 5))
    print('max_run', max(histogram))
    print('distinct_pairs', distinct_pairs, 'codes_per_pair', rows / distinct_pairs)
    print('runs_with_repeated_codes', repeated_runs)
    print('uniform_runs', uniform_runs, 'uniform_share', uniform_runs / runs)
