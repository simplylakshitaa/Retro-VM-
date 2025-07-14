import sys
import joblib
import pandas as pd

def extract_features(ssid):
    return {
        "length": len(ssid),
        "has_special": int(any(c in "!@#$%^&*()-_=+[{]}\\|;:'\",<.>/?`~" for c in ssid)),
        "has_digit": int(any(c.isdigit() for c in ssid)),
        "has_upper": int(any(c.isupper() for c in ssid)),
        "is_common": int(ssid.lower() in ["home", "wifi", "default", "netgear", "tp-link"]),
        "is_manufacturer": int(any(keyword in ssid.lower() for keyword in ["tplink", "dlink", "netgear"]))
    }

if len(sys.argv) < 2:
    print("Usage: python predict_ssid.py <SSID>")
    sys.exit(1)

ssid = sys.argv[1]
features = extract_features(ssid)

model = joblib.load("src/models/ssid_model.pkl")

# Convert dict to DataFrame with correct columns
df = pd.DataFrame([features])

prediction = model.predict(df)[0]

if prediction == 1:
    print(f"SSID '{ssid}' is likely weak or default.")
else:
    print(f"SSID '{ssid}' appears secure.")
