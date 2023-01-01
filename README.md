# es-sector-updater
Utility to download latest sectorfile from GNG and update the existing PRF and ASR files to point to it.

## Configuration
Configuration is done via `config.json`, all of the settings are required. Included is the default configuration (simply put: my configuration).

- `fir` - FIR identifier, GNG places all of the ASRs in a folder named after the FIR,
- `package_name` - string, that the sector file entry at the download website has to contain in order for the program to recognize it as the correct sector link,
- `es_path` - path to EuroScope data folder, usually it's path to My Documents\Euroscope,
- `prf_prefix` - string, that the program will use to determine if the PRF file is to be updated. Used to avoid updating the profile files of othe packages.
