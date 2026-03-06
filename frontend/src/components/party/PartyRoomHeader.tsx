import Image from 'next/image'
import { SkipForward, Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { PartyHeader } from './PartyHeader'
import { PreferencesDialog } from '@/components/preferences/PreferencesDialog'
import { PartyResponse } from '@/model/partyResponse'

export interface PartyRoomHeaderProps {
    party: PartyResponse
    isLeader: boolean
    isManualPending: boolean
    advanceButtonText?: string | null
    onAdvanceClick: () => void
}

export function PartyRoomHeader({
    party,
    isLeader,
    isManualPending,
    advanceButtonText,
    onAdvanceClick
}: Readonly<PartyRoomHeaderProps>) {
    return (
        <header className="flex flex-col items-center mb-6">
            <div className="flex flex-row items-center mb-2 gap-2 relative w-full justify-center">
                <Image src="/Logo.png" alt="Logo" width={32} height={32} priority />
                <h1 className="text-2xl font-bold tracking-tight text-white">Party Room</h1>

                {isLeader && advanceButtonText && (
                    <div className="absolute right-0">
                        <Button
                            variant="ghost"
                            size="icon"
                            disabled={isManualPending}
                            className="text-zinc-500 hover:text-white hover:bg-zinc-800 transition-colors"
                            onClick={onAdvanceClick}
                            title={advanceButtonText ?? "Advance Phase"}
                        >
                            <SkipForward className="h-5 w-5" />
                        </Button>
                    </div>
                )}
            </div>
            <div className="absolute top-4 left-4 z-50">
                <PreferencesDialog
                    trigger={
                        <Button
                            variant="ghost"
                            size="icon"
                            className="text-zinc-500 hover:text-white hover:bg-zinc-800"
                        >
                            <Settings className="h-5 w-5" />
                            <span className="sr-only">Settings</span>
                        </Button>
                    }
                />
            </div>
            <PartyHeader partyCode={party.code} />
            <div className="mt-2 text-zinc-500 text-sm uppercase tracking-wider font-medium">
                {party.state} Phase
            </div>
        </header>
    )
}
