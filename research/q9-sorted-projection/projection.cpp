#include <algorithm>
#include <array>
#include <atomic>
#include <cstdint>
#include <climits>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fcntl.h>
#include <sys/stat.h>
#include <thread>
#include <unistd.h>
#include <utility>
#include <vector>
#include <unordered_map>

struct __attribute__((packed)) Row {
  std::int64_t user;
  std::uint16_t region;
};
static_assert(sizeof(Row) == 10);
constexpr std::size_t kBlock = 8192;

int build(const char* input, const char* output, const char* dictionary) {
  FILE* in = std::fopen(input, "r");
  FILE* out = std::fopen(output, "wb");
  if (!in || !out) return 1;
  std::unordered_map<std::int32_t, std::uint16_t> codes;
  std::vector<std::int32_t> regions;
  std::array<Row, kBlock> rows;
  std::size_t used = 0;
  std::uint64_t total = 0;
  std::int64_t previous_user = 0;
  bool have_previous_user = false;
  char* line = nullptr;
  std::size_t capacity = 0;
  while (getline(&line, &capacity, in) >= 0) {
    char* end = nullptr;
    const auto user = std::strtoll(line, &end, 10);
    if (*end != '|') return 2;
    if (have_previous_user && user < previous_user) return 10;
    previous_user = user;
    have_previous_user = true;
    const auto region = std::strtol(end + 1, &end, 10);
    if (*end != '\n' && *end != '\0') return 3;
    if (region < INT32_MIN || region > INT32_MAX) return 4;
    const auto region_value = static_cast<std::int32_t>(region);
    const auto found = codes.find(region_value);
    std::uint16_t code;
    if (found == codes.end()) {
      if (regions.size() >= UINT16_MAX) return 5;
      code = static_cast<std::uint16_t>(regions.size());
      codes.emplace(region_value, code);
      regions.push_back(region_value);
    } else {
      code = found->second;
    }
    rows[used++] = Row{user, code};
    if (used == kBlock) {
      if (std::fwrite(rows.data(), sizeof(Row), used, out) != used) return 6;
      total += used;
      used = 0;
    }
  }
  if (used && std::fwrite(rows.data(), sizeof(Row), used, out) != used) return 7;
  total += used;
  std::free(line);
  std::fclose(in);
  std::fclose(out);
  FILE* dict = std::fopen(dictionary, "wb");
  if (!dict) return 8;
  const std::uint32_t count = regions.size();
  if (std::fwrite(&count, sizeof(count), 1, dict) != 1 ||
      std::fwrite(regions.data(), sizeof(std::int32_t), regions.size(), dict) != regions.size())
    return 9;
  std::fclose(dict);
  std::fprintf(stderr, "rows=%llu regions=%u\n", static_cast<unsigned long long>(total), count);
  return 0;
}

int scan(const char* input, const char* dictionary, std::size_t degree) {
  if (!degree || degree > 64) return 1;
  FILE* dict = std::fopen(dictionary, "rb");
  if (!dict) return 2;
  std::uint32_t count = 0;
  if (std::fread(&count, sizeof(count), 1, dict) != 1 || count > UINT16_MAX) return 3;
  std::vector<std::int32_t> regions(count);
  if (std::fread(regions.data(), sizeof(std::int32_t), count, dict) != count) return 4;
  std::fclose(dict);

  const int fd = open(input, O_RDONLY);
  if (fd < 0) return 5;
  struct stat statbuf {};
  if (fstat(fd, &statbuf) != 0 || statbuf.st_size % sizeof(Row) != 0) return 6;
  const std::size_t total = statbuf.st_size / sizeof(Row);
  std::vector<std::size_t> boundaries(degree + 1);
  boundaries[degree] = total;
  for (std::size_t part = 1; part < degree; ++part) {
    std::size_t at = total * part / degree;
    Row previous {};
    if (pread(fd, &previous, sizeof(Row), (at - 1) * sizeof(Row)) != sizeof(Row)) return 7;
    while (at < total) {
      Row here {};
      if (pread(fd, &here, sizeof(Row), at * sizeof(Row)) != sizeof(Row)) return 8;
      if (here.user != previous.user) break;
      ++at;
    }
    boundaries[part] = at;
  }
  struct Local {
    std::vector<std::uint32_t> marks;
    std::vector<std::uint32_t> counts;
    explicit Local(std::size_t groups) : marks(groups), counts(groups) {}
  };
  std::vector<Local> locals;
  locals.reserve(degree);
  for (std::size_t part = 0; part < degree; ++part) locals.emplace_back(count);
  std::atomic<int> error = 0;
  std::vector<std::thread> threads;
  for (std::size_t part = 0; part < degree; ++part) {
    threads.emplace_back([&, part] {
      std::array<Row, kBlock> rows;
      auto& local = locals[part];
      std::size_t at = boundaries[part];
      std::uint32_t epoch = 0;
      std::int64_t previous = 0;
      bool have_previous = false;
      while (at < boundaries[part + 1]) {
        const std::size_t wanted = std::min(kBlock, boundaries[part + 1] - at);
        const ssize_t got = pread(fd, rows.data(), wanted * sizeof(Row), at * sizeof(Row));
        if (got <= 0 || got % sizeof(Row) != 0) { error = 9; return; }
        const std::size_t received = got / sizeof(Row);
        for (std::size_t row_at = 0; row_at < received; ++row_at) {
          const Row row = rows[row_at];
          if (row.region >= count) { error = 10; return; }
          if (!have_previous || row.user != previous) {
            ++epoch;
            previous = row.user;
            have_previous = true;
          }
          if (local.marks[row.region] != epoch) {
            local.marks[row.region] = epoch;
            ++local.counts[row.region];
          }
        }
        at += received;
      }
    });
  }
  for (auto& thread : threads) thread.join();
  close(fd);
  if (error) return error;
  std::vector<std::pair<std::uint32_t, std::int32_t>> ranked;
  for (std::size_t code = 0; code < count; ++code) {
    std::uint32_t value = 0;
    for (auto& local : locals) value += local.counts[code];
    if (value) ranked.emplace_back(value, regions[code]);
  }
  std::partial_sort(ranked.begin(), ranked.begin() + std::min<std::size_t>(10, ranked.size()),
                    ranked.end(), [](auto a, auto b) {
                      return a.first > b.first || (a.first == b.first && a.second < b.second);
                    });
  for (std::size_t at = 0; at < std::min<std::size_t>(10, ranked.size()); ++at)
    std::printf("%d,%u\n", ranked[at].second, ranked[at].first);
  return 0;
}

int main(int argc, char** argv) {
  if (argc != 5) return 1;
  if (std::strcmp(argv[1], "build") == 0) return build(argv[2], argv[3], argv[4]);
  if (std::strcmp(argv[1], "scan") == 0) return scan(argv[2], argv[3], std::strtoul(argv[4], nullptr, 10));
  return 1;
}
