'use client'

import { MemberInfo } from '@/model'
import { PartyMemberCard } from './PartyMemberCard'

interface PartyMemberListProps {
  members: MemberInfo[]
  loading: boolean
  currentUserId: string
  isCurrentUserLeader: boolean
  onKick: (memberId: string) => void
  onPromote: (memberId: string) => void
}

export function PartyMemberList({
  members,
  loading,
  currentUserId,
  isCurrentUserLeader,
  onKick,
  onPromote
}: PartyMemberListProps) {
  if (loading && members.length === 0) {
    return (
      <div className="flex flex-col gap-3 w-full">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-16 bg-muted animate-pulse rounded-xl" />
        ))}
      </div>
    )
  }

  if (members.length === 0) {
    return <div className="text-center text-muted-foreground py-8">No members found.</div>
  }

  return (
    <div className="flex flex-col gap-3 w-full pb-20">
      {members
        .sort((a, b) => {
          // Sort Leader first, then current user, then others
          if (a.is_leader) return -1
          if (b.is_leader) return 1
          if (a.user_id === currentUserId) return -1
          if (b.user_id === currentUserId) return 1
          return 0
        })
        .map((member) => (
          <PartyMemberCard
            key={member.user_id}
            member={member}
            isCurrentUserLeader={isCurrentUserLeader}
            isCurrentUser={member.user_id === currentUserId}
            onKick={onKick}
            onPromote={onPromote}
          />
        ))}
    </div>
  )
}
