#!/usr/bin/env python3

# Script taken from https://brianli.com/2021/06/how-to-find-broken-links-with-python/
# and adjusted.

import sys
import os
import requests
from bs4 import BeautifulSoup
from concurrent.futures import ThreadPoolExecutor

def get_broken_links(path):
    f = open(path,"r")
    data = f.read()

    # Parse HTML.
    soup = BeautifulSoup(data, features="html.parser")

    # Filter links which interest us.
    def _filter(elem):
        return 'cosmos' in elem['href']

    # Create a list containing all links
    links = [link.get("href") for link in filter(_filter, soup.find_all("a", href=True))]
    print(links)

    # Initialize list for broken links.
    broken_links = []

    # Internal function for validating HTTP status code.
    def _validate_url(url):
        r = requests.get(url)
        page = BeautifulSoup(r.content, features="html.parser")
        # TODO: doesn't yet work (?)
        hs = page.find_all("div", attrs={'class':'h1'})
        print(hs)

        def _filter(h):
            "Page Not Found" in h.contents

        pageNotFound = len(filter(_filter, hs)) > 0

        if r.status_code == 404 or pageNotFound:
            broken_links.append(url)

    # Loop through links checking for 404 responses, and append to list.
    with ThreadPoolExecutor(max_workers=8) as executor:
        executor.map(_validate_url, links)

    return broken_links

doc_folder = 'target/doc/'

def check_project(project):
    project_path = doc_folder + project
    broken_links = {}

    for dirName, subdirList, fileList in os.walk(project_path):
        for fname in fileList:
            if fname.endswith(".html"):
                fpath = dirName + '/' + fname

                file_broken_links = get_broken_links(fpath)
                if len(file_broken_links) > 0:
                    broken_links[fpath] = file_broken_links

    return broken_links

# main

broken_links = {}
projects = ['test']

for project in projects:
    broken_links.update(check_project(project))

if len(broken_links) > 0:
    print("Dead links found!")
    for fpath, links in broken_links.items():
        print("In ", fpath)
        for link in links:
            print("  ", link)
    sys.exit(1)
