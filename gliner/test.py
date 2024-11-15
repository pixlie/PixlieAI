from gliner import GLiNER

# Initialize GLiNER with the base model
model = GLiNER.from_pretrained("urchade/gliner_mediumv2.1")

# Sample text for entity prediction
text = """
List of funded startups for 2024
"""

# Labels for entity prediction
# Most GLiNER models should work best when entity types are in lower case or title case
labels = [
    "StartupWebsite",
    "CompanyWebsite",
    "FounderProfile",
    "FundingNews",
    "InvestorWebsite",
    "InvestorProfile"
]

# Perform entity prediction
entities = model.predict_entities(text, labels, threshold=0.5)

# Display predicted entities and their labels
for entity in entities:
    print(entity["text"], "=>", entity["label"])
