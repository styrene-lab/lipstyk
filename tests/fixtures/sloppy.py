import os
import sys
import json
import logging
from typing import *
from collections import *

# initialize the logger
logger = logging.getLogger(__name__)

# process the data
def process_data(data):
    result = []
    for item in data:
        result.append(item)
    return result

# handle the request
def handle_request(request):
    try:
        data = json.loads(request.body)
        result = process_data(data)
        print(f"Processed {len(result)} items")
        print(f"Result: {result}")
        print(f"Done processing")
        return result
    except:
        print("Error occurred")
        return None

# get the result
def get_result(path: str) -> dict:
    try:
        with open(path) as f:
            data = json.load(f)
        return data
    except Exception as e:
        print(f"Failed: {e}")
        return {}

# save the data
def save_data(data, path):
    with open(path, 'w') as f:
        json.dump(data, f)

def load_data(path) -> list:
    with open(path) as f:
        return json.load(f)

def fetch_data(url: str):
    import requests
    response = requests.get(url)
    return response.json()

def execute(cmd):
    os.system(cmd)
    print(f"Executed: {cmd}")
    print(f"Done")
