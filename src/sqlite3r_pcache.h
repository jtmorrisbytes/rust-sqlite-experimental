/* sqlite3r_pcache.h */
#ifndef SQLITE3R_PCACHE_H
#define SQLITE3R_PCACHE_H

#include <stddef.h>

#define SQLITE3R_FLAG_DIRTY 1

/* Core Engine Hooks */
int           sqlite3r_pcache_init_thread(size_t initial_size);
unsigned char* sqlite3r_pcache_fetch(size_t page_id);
void           sqlite3r_pcache_set_flag(size_t page_id, unsigned char flag);
void           sqlite3r_pcache_clear_flag(size_t page_id, unsigned char flag);
int            sqlite3r_pcache_has_flag(size_t page_id, unsigned char flag);
size_t         sqlite3r_pcache_get_capacity(void);
void           sqlite3r_pcache_destroy_thread(void);

#endif
