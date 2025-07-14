import pandas as pd
from sklearn.ensemble import RandomForestClassifier
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score
import joblib
import re
import os
import nltk
from nltk.corpus import stopwords
from nltk.tokenize import word_tokenize
from nltk.stem import WordNetLemmatizer

# Initialize NLP tools
stop_words = set(stopwords.words('english'))
lemmatizer = WordNetLemmatizer()

def preprocess_text(text):
    """Advanced text preprocessing"""
    # Lowercase
    text = text.lower()
    # Remove special chars
    text = re.sub(r'[^a-z0-9\s]', '', text)
    # Tokenize
    tokens = word_tokenize(text)
    # Remove stopwords and lemmatize
    tokens = [lemmatizer.lemmatize(token) for token in tokens if token not in stop_words]
    return ' '.join(tokens)

def create_threat_dataset():
    """Create realistic threat dataset with 100 samples"""
    data = []
    
    # Malicious samples (50)
    for i in range(25):
        data.append({
            'name': f'malicious_{i}.xpi',
            'permissions': 'tabs,debugger,webRequest,downloads',
            'description': 'steals user data and credentials',
            'label': 1
        })
        data.append({
            'name': f'keylogger_{i}.dll',
            'permissions': 'input,processes,system',
            'description': 'records all keyboard inputs',
            'label': 1
        })
    
    # Benign samples (50)
    for i in range(25):
        data.append({
            'name': f'adblock_{i}.xpi',
            'permissions': 'webRequest,storage',
            'description': 'blocks advertisements on web pages',
            'label': 0
        })
        data.append({
            'name': f'pdfviewer_{i}.so',
            'permissions': 'pdfs,printing',
            'description': 'views pdf documents in browser',
            'label': 0
        })
    
    df = pd.DataFrame(data)
    # Create combined text features
    df['combined'] = df['name'] + ' ' + df['permissions'] + ' ' + df['description']
    df['combined'] = df['combined'].apply(preprocess_text)
    df.to_csv("models/threat_dataset.csv", index=False)
    return df

def train_threat_model():
    """Train advanced threat detection model with NLP"""
    df = create_threat_dataset()
    
    # Feature engineering
    vectorizer = TfidfVectorizer(
        ngram_range=(1, 3),
        max_features=1000,
        stop_words='english'
    )
    
    X = vectorizer.fit_transform(df['combined'])
    y = df['label']
    
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.3, random_state=42
    )
    
    # Train model
    model = RandomForestClassifier(
        n_estimators=200,
        max_depth=10,
        class_weight='balanced',
        random_state=42
    )
    model.fit(X_train, y_train)
    
    # Evaluate
    y_pred = model.predict(X_test)
    accuracy = accuracy_score(y_test, y_pred)
    
    # Save artifacts
    joblib.dump(model, "models/threat_model.pkl")
    joblib.dump(vectorizer, "models/threat_vectorizer.pkl")
    
    print(f"\nThreat Detection Model Evaluation:")
    print(f"Accuracy: {accuracy:.2%}")
    print(f"Features used: {X.shape[1]}")
    print("Model saved with vectorizer")

def create_ssid_dataset():
    """Create labeled SSID dataset for model training"""
    ssids = []
    common_names = {"home", "wifi", "default", "netgear", "linksys", "tp-link"}
    manufacturer_prefixes = {"TP-LINK", "NETGEAR", "D-LINK", "MI", "JioFi", "Tenda"}

    # Weak SSIDs
    weak_examples = [
        "HOME123", "wifi2020", "TP-LINK123", "netgear", "admin1", "password123", "jiofi2023"
    ]
    
    for ssid in weak_examples:
        ssids.append({
            "ssid": ssid,
            "length": len(ssid),
            "has_special": int(bool(re.search(r"[^a-zA-Z0-9]", ssid))),
            "has_digit": int(any(c.isdigit() for c in ssid)),
            "has_upper": int(any(c.isupper() for c in ssid)),
            "is_common": int(ssid.lower() in common_names),
            "is_manufacturer": int(any(ssid.upper().startswith(prefix) for prefix in manufacturer_prefixes)),
            "label": 1  # weak
        })

    # Strong SSIDs
    strong_examples = [
        "Phoenix_9823", "SecureNet_77", "MyPrivateLAN", "OrangeJuice_X", "X1z9_aQ#1"
    ]
    
    for ssid in strong_examples:
        ssids.append({
            "ssid": ssid,
            "length": len(ssid),
            "has_special": int(bool(re.search(r"[^a-zA-Z0-9]", ssid))),
            "has_digit": int(any(c.isdigit() for c in ssid)),
            "has_upper": int(any(c.isupper() for c in ssid)),
            "is_common": int(ssid.lower() in common_names),
            "is_manufacturer": int(any(ssid.upper().startswith(prefix) for prefix in manufacturer_prefixes)),
            "label": 0  # strong
        })

    return pd.DataFrame(ssids)
def train_ssid_model():
    """Train and save SSID strength classifier"""
    df = create_ssid_dataset()
    features = ['length', 'has_special', 'has_digit', 'has_upper', 'is_common', 'is_manufacturer']
    
    X = df[features]
    y = df['label']
    
    model = RandomForestClassifier(n_estimators=100, random_state=42)
    model.fit(X, y)
    
    joblib.dump(model, "models/ssid_model.pkl")
    print("SSID model trained and saved")


if __name__ == "__main__":
    print("Training SSID strength model...")
    train_ssid_model()
    
    print("\nTraining advanced threat detection model with NLP...")
    train_threat_model()
