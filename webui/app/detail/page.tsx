"use client";

import { useSearchParams } from 'next/navigation'

export default function Detail() {
    const searchParams = useSearchParams()
    const imageId = searchParams.get('image_id')
    const textIds = searchParams.get('text_ids')
    return (
        <main>
            <div className="container m-auto">
                <div className="8">
                    <img
                        src={`/api/image?image_id=${imageId}&text_ids=${textIds}`}
                    >
                    </img>
                </div>

            </div>
        </main>
    )
}
