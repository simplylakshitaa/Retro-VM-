import joblib
import pandas as pd
import sys
import re
import traceback
from sklearn.feature_extraction.text import TfidfVectorizer

if len(sys.argv) < 2:
    print("Usage: python predict_extension.py <extension or domain>")
    sys.exit(1)

input_str = sys.argv[1].lower()

# Load model and vectorizer with proper error handling
try:
    model = joblib.load("src/models/threat_model.pkl")
    vectorizer = joblib.load("src/models/threat_vectorizer.pkl")
except Exception as e:
    print(f"❌ Failed to load model: {str(e)}")
    traceback.print_exc()
    sys.exit(1)

def detect_type(name):
    """Improved type detection with more extensions"""
    ext_pattern = r"\.(xpi|dll|so|exe|crx|js|py|jar|bin|sh|bat|cmd)$"
    domain_pattern = r"\.(com|org|net|xyz|tech|site|io|gov|edu|info|biz|co|us|uk|ca)$"
    
    if re.search(ext_pattern, name, re.IGNORECASE):
        return "extension"
    elif re.search(domain_pattern, name, re.IGNORECASE):
        return "domain"
    return "unknown"

def preprocess_input(text, input_type):
    """Prepare input based on detected type"""
    if input_type == "extension":
        return f"{text} extension plugin addon browser chrome firefox"
    elif input_type == "domain":
        return f"{text} website url domain internet web"
    return f"{text} unknown suspicious threat"

detected_type = detect_type(input_str)
processed_text = preprocess_input(input_str, detected_type)

# Debug info
print(f"Detected type: {detected_type}")
print(f"Processed text: {processed_text}")

# Predict with robust input handling
try:
    # Ensure consistent input format (always use list)
    X = vectorizer.transform([processed_text])
    
    # Get both prediction and probability
    prediction = model.predict(X)[0]
    proba = model.predict_proba(X)[0][1]  # Probability of being malicious
    
    # Format results
    result = {
        'input': input_str,
        'type': detected_type,
        'prediction': 'malicious' if prediction == 1 else 'safe',
        'confidence': f"{proba*100:.1f}%",
        'details': {
            'model_type': type(model).__name__,
            'features_used': X.shape[1]
        }
    }
    
    # Print JSON-formatted result
    import json
    print(json.dumps(result, indent=2))
    
except Exception as e:
    print(f"❌ Prediction failed: {str(e)}")
    traceback.print_exc()
    sys.exit(1)