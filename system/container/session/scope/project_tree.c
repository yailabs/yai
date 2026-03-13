#include <stdio.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>

/**
 * YAI Project Tree Scanner (RFC-YAI-003)
 * Scansiona il Workspace senza saturare la RAM.
 */
void yai_scan_workspace(const char *path, int depth) {
    DIR *dir;
    struct dirent *entry;
    struct stat entry_stat;
    char full_path[1024];

    // Limite di sicurezza (I-002: Determinismo)
    if (depth > 5) return; 

    if (!(dir = opendir(path))) {
        return; // Workspace non accessibile o rimosso
    }

    while ((entry = readdir(dir)) != NULL) {
        // Ignoriamo le entry nascoste e i puntatori di sistema
        if (entry->d_name[0] == '.') continue;

        // Costruiamo il path completo in modo sicuro
        snprintf(full_path, sizeof(full_path), "%s/%s", path, entry->d_name);
        
        // Otteniamo i metadati del file
        if (stat(full_path, &entry_stat) == -1) continue;

        // Indentazione per visualizzare l'albero (Simulazione logica)
        for (int i = 0; i < depth; i++) printf("  ");

        if (S_ISDIR(entry_stat.st_mode)) {
            printf("[FOLDER] %s\n", entry->d_name);
            // Ricorsione controllata
            yai_scan_workspace(full_path, depth + 1);
        } else {
            printf("[FILE]   %s (%lld bytes)\n", entry->d_name, (long long)entry_stat.st_size);
        }
    }
    closedir(dir);
}