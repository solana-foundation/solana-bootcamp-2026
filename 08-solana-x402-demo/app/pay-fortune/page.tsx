import Link from 'next/link'

export default function PayFortunePage() {

  return (
    <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-[#14F195] to-[#9945FF]">
      <div className="bg-white rounded-2xl shadow-2xl p-12 text-center">
        <h2 className="text-2xl font-bold text-gray-800 mb-4">✅ Payment Successful!</h2>
        <p className="text-gray-700 mb-6">Click the button below to access your fortune teller:</p>

        <div className="space-y-4">
          <Link
            href="/fortune"
            target="_blank"
            className="block w-full px-6 py-3 bg-purple-600 text-white rounded-lg font-semibold hover:bg-purple-700 transition-colors"
          >
            Open Fortune Teller
          </Link>

          <Link
            href="/"
            className="block w-full px-6 py-3 bg-neutral-800 text-white rounded-lg font-semibold hover:opacity-90 transition-opacity"
          >
            Back to Home
          </Link>
        </div>
      </div>
    </div>
  )
}
