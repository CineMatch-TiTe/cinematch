import React from 'react'
import Image from 'next/image'

interface AuthLayoutProps {
  title: string
  subtitle: string
  children: React.ReactNode
}

export function AuthLayout({ title, subtitle, children }: Readonly<AuthLayoutProps>) {
  return (
    <main className="relative z-10 w-full max-w-md px-6">
      <div className="flex flex-col items-center justify-center mb-8 gap-4 text-center">
        <Image
          src="/Logo.png"
          className="w-36 h-auto"
          alt="CineMatch"
          width={320}
          height={320}
          loading="eager"
        />
        <h1 className="text-4xl font-bold tracking-tighter text-white mb-2">{title}</h1>
        <p className="text-zinc-400">{subtitle}</p>
      </div>
      
      {children}
    </main>
  )
}
