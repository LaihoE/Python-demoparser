from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def coordinates(file):
    parser = DemoParser(file)
    game_events = parser.parse_events("")
    names = []
    for event in game_events:
        names.append(event.name)
    return names


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    
    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))

    # Combine all lists
    all_names = []
    for name_list in results:
        all_names.extend(name_list)