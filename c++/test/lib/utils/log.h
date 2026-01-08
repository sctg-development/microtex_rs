#ifndef LOG_H_INCLUDED
#define LOG_H_INCLUDED

#ifdef HAVE_LOG

#include <cstdio>
#include <cstdlib>
#include <string>

// clang-format off
#define ANSI_BOLD        "\x1b[1m"
#define ANSI_COLOR_CYAN  "\x1b[36m"
#define ANSI_COLOR_RED   "\x1b[31m"
#define ANSI_COLOR_GREEN "\x1b[32m"
#define ANSI_UNDERLINE   "\x1b[4m"
#define ANSI_RESET       "\x1b[0m"
// clang-format on

// Log levels: 0=off, 1=error, 2=warn, 3=info, 4=debug
static inline int _get_log_level() {
  const char* level_env = std::getenv("RUST_LOG");
  if (!level_env) {
    return 2;  // Default: warn level
  }
  
  std::string level_str(level_env);
  
  // Check if "debug" is in the string
  if (level_str.find("debug") != std::string::npos || level_str.find("microtex=debug") != std::string::npos) {
    return 4;  // Debug level
  }
  if (level_str.find("info") != std::string::npos) {
    return 3;  // Info level
  }
  if (level_str.find("warn") != std::string::npos) {
    return 2;  // Warn level
  }
  if (level_str.find("error") != std::string::npos) {
    return 1;  // Error level
  }
  if (level_str.find("off") != std::string::npos) {
    return 0;  // Off level
  }
  
  return 2;  // Default: warn level
}

#define dbg(format, ...)                                                              \
  do {                                                                               \
    if (_get_log_level() >= 1) {                                                    \
      fprintf(                                                                       \
        stdout,                                                                      \
        "FILE: " ANSI_UNDERLINE "%s" ANSI_RESET ", LINE: " ANSI_COLOR_RED "%d" ANSI_RESET \
        ", FUNCTION: " ANSI_COLOR_CYAN "%s" ANSI_RESET ", MSG: " format,            \
        __FILE__,                                                                    \
        __LINE__,                                                                    \
        __FUNCTION__,                                                                \
        ##__VA_ARGS__                                                                \
      );                                                                             \
    }                                                                                \
  } while (0)

#define logv(format, ...) \
  do { \
    if (_get_log_level() >= 4) { \
      fprintf(stdout, format, ##__VA_ARGS__); \
    } \
  } while (0)

#define loge(format, ...) \
  do { \
    if (_get_log_level() >= 1) { \
      fprintf(stderr, format, ##__VA_ARGS__); \
    } \
  } while (0)

#endif  // HAVE_LOG

#endif
