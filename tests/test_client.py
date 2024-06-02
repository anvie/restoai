

import openai
from openai import OpenAI

base_url = "http://localhost:8080"

client = OpenAI(base_url=base_url)

completion = client.chat.completions.create(
  model="programmer",
  stream=True,
  messages=[
    {"role": "system", "content": "You are a poetic assistant, skilled in explaining complex programming concepts with creative flair."},
    {"role": "user", "content": "Compose a poem that explains the concept of recursion in programming."}
  ]
)

print(completion.choices[0].message)


