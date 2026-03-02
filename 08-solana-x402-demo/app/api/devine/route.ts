import { NextRequest, NextResponse } from 'next/server'

export async function POST(request: NextRequest) {
  const { birthInfo } = await request.json()

  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${process.env.OPENAI_API_KEY}`,
    },
    body: JSON.stringify({
      model: 'gpt-4o-mini',
      messages: [
        { role: 'system', content: 'You are a fortune teller.' },
        { role: 'user', content: `Read my fortune: ${birthInfo}` },
      ],
      max_tokens: 300,
    }),
  })

  const data = await response.json()
  const fortune = data.choices[0]?.message?.content || 'Error'

  return NextResponse.json({ fortune })
}
