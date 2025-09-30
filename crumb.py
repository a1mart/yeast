import requests
import re
import json
import time
from urllib.parse import unquote

def get_yahoo_session_and_crumb(symbol="TSLA"):
    """
    Returns a requests.Session object and a valid Yahoo Finance crumb
    """
    session = requests.Session()
    
    # More realistic headers
    session.headers.update({
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                     "AppleWebKit/537.36 (KHTML, like Gecko) "
                     "Chrome/120.0.0.0 Safari/537.36",
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.5",
        "Accept-Encoding": "gzip, deflate, br",
        "DNT": "1",
        "Connection": "keep-alive",
        "Upgrade-Insecure-Requests": "1",
        "Sec-Fetch-Dest": "document",
        "Sec-Fetch-Mode": "navigate",
        "Sec-Fetch-Site": "none",
        "Cache-Control": "max-age=0"
    })
    
    # Step 1: Visit main page first to establish session
    main_url = "https://finance.yahoo.com/"
    session.get(main_url)
    time.sleep(1)  # Brief delay
    
    # Step 2: Get the quote page HTML
    url = f"https://finance.yahoo.com/quote/{symbol}"
    resp = session.get(url)
    
    if resp.status_code != 200:
        raise ValueError(f"Failed to fetch page. Status code: {resp.status_code}")
    
    html = resp.text
    
    # Debug: Check if we got the actual page content
    if "Yahoo is part of" in html or "blocked" in html.lower():
        raise ValueError("Page appears to be blocked or redirected")
    
    # Step 3: Multiple regex patterns to find crumb
    crumb_patterns = [
        r'"CrumbStore":\{"crumb":"([^"]+)"\}',
        r'CrumbStore":\{"crumb":"([^"]+)"\}',
        r'"crumb":"([^"]+)"',
        r'crumb["\']?:\s*["\']([^"\']+)["\']'
    ]
    
    crumb = None
    for pattern in crumb_patterns:
        m = re.search(pattern, html)
        if m:
            crumb = m.group(1)
            break
    
    if not crumb:
        # Try to find it in script tags
        script_matches = re.findall(r'<script[^>]*>(.*?)</script>', html, re.DOTALL)
        for script in script_matches:
            for pattern in crumb_patterns:
                m = re.search(pattern, script)
                if m:
                    crumb = m.group(1)
                    break
            if crumb:
                break
    
    if not crumb:
        raise ValueError("Could not find crumb in page. Yahoo may have changed their page structure.")
    
    # Decode the crumb properly
    try:
        # Handle Unicode escapes
        crumb = crumb.encode('utf-8').decode('unicode_escape')
        # Handle URL encoding
        crumb = unquote(crumb)
    except Exception as e:
        print(f"Warning: Could not decode crumb properly: {e}")
        # Use raw crumb if decoding fails
        pass
    
    print(f"Found crumb: {crumb}")
    return session, crumb

def call_quote_summary(symbol="TSLA"):
    """
    Get quote summary data from Yahoo Finance
    """
    try:
        session, crumb = get_yahoo_session_and_crumb(symbol)
        
        # Build API URL
        modules = "assetProfile,financialData,defaultKeyStatistics,summaryDetail,price,summaryProfile"
        api_url = f"https://query1.finance.yahoo.com/v10/finance/quoteSummary/{symbol}"
        
        params = {
            'modules': modules,
            'crumb': crumb
        }
        
        # Make the API request
        response = session.get(api_url, params=params)
        
        if response.status_code != 200:
            raise ValueError(f"API request failed with status: {response.status_code}")
        
        data = response.json()
        
        # Check for API errors
        if 'quoteSummary' not in data or data['quoteSummary'].get('error'):
            error_msg = data.get('quoteSummary', {}).get('error', {}).get('description', 'Unknown API error')
            raise ValueError(f"Yahoo Finance API error: {error_msg}")
        
        return data
        
    except Exception as e:
        print(f"Error: {e}")
        return None

def alternative_approach(symbol="TSLA"):
    """
    Alternative approach using different endpoint
    """
    session = requests.Session()
    session.headers.update({
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                     "AppleWebKit/537.36 (KHTML, like Gecko) "
                     "Chrome/120.0.0.0 Safari/537.36"
    })
    
    # Try the chart API which sometimes doesn't need a crumb
    chart_url = f"https://query1.finance.yahoo.com/v8/finance/chart/{symbol}"
    
    try:
        response = session.get(chart_url)
        if response.status_code == 200:
            return response.json()
        else:
            print(f"Chart API failed with status: {response.status_code}")
            return None
    except Exception as e:
        print(f"Alternative approach failed: {e}")
        return None

if __name__ == "__main__":
    symbol = "TSLA"
    
    print("Trying main approach...")
    result = call_quote_summary(symbol)
    
    if result:
        print("Success with main approach!")
        print(json.dumps(result, indent=2)[:500] + "...")  # Print first 500 chars
    else:
        print("\nTrying alternative approach...")
        result = alternative_approach(symbol)
        if result:
            print("Success with alternative approach!")
            print(json.dumps(result, indent=2)[:500] + "...")
        else:
            print("Both approaches failed. Yahoo may be blocking requests.")